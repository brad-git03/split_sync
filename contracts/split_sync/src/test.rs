#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, vec};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient as TokenAdminClient};

// Helper function to set up a mock token environment
fn create_token<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = env.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(env, &contract_address),
        TokenAdminClient::new(env, &contract_address),
    )
}

// Test 1 (Happy path): The MVP transaction executes successfully end-to-end
#[test]
fn test_splitsync_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let client_wallet = Address::generate(&env);
    let dev_wallet = Address::generate(&env);
    let designer_wallet = Address::generate(&env);
    let token_admin = Address::generate(&env);
    
    let (token, admin_client) = create_token(&env, &token_admin);
    admin_client.mint(&client_wallet, &1000);

    let contract_id = env.register_contract(None, SplitSyncContract);
    let contract_client = SplitSyncContractClient::new(&env, &contract_id);

    // Initialize a 70/30 split
    let shares = vec![
        &env,
        Share { recipient: dev_wallet.clone(), basis_points: 7000 },
        Share { recipient: designer_wallet.clone(), basis_points: 3000 },
    ];
    contract_client.init(&shares);

    // Client pays 1000 USDC
    contract_client.pay(&token.address, &client_wallet, &1000);

    // Verify correct split distributions
    assert_eq!(token.balance(&dev_wallet), 700);
    assert_eq!(token.balance(&designer_wallet), 300);
    assert_eq!(token.balance(&client_wallet), 0);
    assert_eq!(token.balance(&contract_id), 0); // Contract holds nothing
}

// Test 2 (Edge case): Initialization fails if basis points do not equal 10,000
#[test]
#[should_panic(expected = "Shares must total exactly 10000 basis points (100%)")]
fn test_init_invalid_basis_points() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SplitSyncContract);
    let contract_client = SplitSyncContractClient::new(&env, &contract_id);

    let dev_wallet = Address::generate(&env);
    let designer_wallet = Address::generate(&env);

    // Attempting a 70/40 split (11,000 bp)
    let shares = vec![
        &env,
        Share { recipient: dev_wallet, basis_points: 7000 },
        Share { recipient: designer_wallet, basis_points: 4000 },
    ];
    
    contract_client.init(&shares);
}

// Test 3 (Edge case): Unauthorized caller cannot force another user to pay
#[test]
#[should_panic]
fn test_unauthorized_payment_fails() {
    let env = Env::default();
    
    let client_wallet = Address::generate(&env);
    let dev_wallet = Address::generate(&env);
    let token_admin = Address::generate(&env);
    
    let (token, admin_client) = create_token(&env, &token_admin);
    
    // We strictly mock auth for minting, but turn it OFF for the payment test
    admin_client.mock_all_auths().mint(&client_wallet, &1000);

    let contract_id = env.register_contract(None, SplitSyncContract);
    let contract_client = SplitSyncContractClient::new(&env, &contract_id);

    let shares = vec![&env, Share { recipient: dev_wallet.clone(), basis_points: 10000 }];
    contract_client.init(&shares);

    // Attempting to pull funds from client_wallet without mocking their authorization
    // This will panic on `sender.require_auth()`
    contract_client.pay(&token.address, &client_wallet, &500);
}

// Test 4 (State verification): Verify balances handle odd amounts without trapping dust
#[test]
fn test_state_verification_odd_amounts() {
    let env = Env::default();
    env.mock_all_auths();

    let client_wallet = Address::generate(&env);
    let dev_wallet = Address::generate(&env);
    let designer_wallet = Address::generate(&env);
    let token_admin = Address::generate(&env);
    
    let (token, admin_client) = create_token(&env, &token_admin);
    admin_client.mint(&client_wallet, &1005); // Odd amount

    let contract_id = env.register_contract(None, SplitSyncContract);
    let contract_client = SplitSyncContractClient::new(&env, &contract_id);

    let shares = vec![
        &env,
        Share { recipient: dev_wallet.clone(), basis_points: 3333 },
        Share { recipient: designer_wallet.clone(), basis_points: 6667 },
    ];
    contract_client.init(&shares);

    contract_client.pay(&token.address, &client_wallet, &1005);

    // 1005 * 0.3333 = 334.96 -> rounds down to 334
    assert_eq!(token.balance(&dev_wallet), 334);
    // 1005 * 0.6667 = 670.03 -> rounds down to 670
    assert_eq!(token.balance(&designer_wallet), 670);
    
    // 1 stroop of dust gets left in the contract due to integer math floor
    assert_eq!(token.balance(&contract_id), 1); 
}

// Test 5 (Edge case): Prevent payment if contract is not initialized
#[test]
#[should_panic(expected = "Contract not initialized")]
fn test_pay_before_init() {
    let env = Env::default();
    env.mock_all_auths();

    let client_wallet = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token, admin_client) = create_token(&env, &token_admin);
    admin_client.mint(&client_wallet, &1000);

    let contract_id = env.register_contract(None, SplitSyncContract);
    let contract_client = SplitSyncContractClient::new(&env, &contract_id);

    // Attempting to pay before calling `init`
    contract_client.pay(&token.address, &client_wallet, &1000);
}
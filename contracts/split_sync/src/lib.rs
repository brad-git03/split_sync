#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

// Represents a single recipient and their percentage of the split
// basis_points: 10000 = 100%, 7000 = 70%, etc.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Share {
    pub recipient: Address,
    pub basis_points: u32,
}

#[contract]
pub struct SplitSyncContract;

const SHARES_KEY: &str = "shares";

#[contractimpl]
impl SplitSyncContract {
    /// Initializes the contract with the split configuration.
    /// Ensures the total percentages equal exactly 100% (10,000 basis points).
    pub fn init(env: Env, shares: Vec<Share>) {
        let mut total = 0;
        for share in shares.iter() {
            total += share.basis_points;
        }
        
        if total != 10000 {
            panic!("Shares must total exactly 10000 basis points (100%)");
        }
        
        env.storage().instance().set(&SHARES_KEY, &shares);
    }

    /// Pulls funds from the sender and instantly distributes them 
    /// to the configured recipients based on their basis points.
    pub fn pay(env: Env, token: Address, sender: Address, amount: i128) {
        // Ensure the sender actually authorized this payment
        sender.require_auth();

        // Retrieve the split configuration
        let shares: Vec<Share> = env
            .storage()
            .instance()
            .get(&SHARES_KEY)
            .expect("Contract not initialized");

        let client = token::Client::new(&env, &token);

        // Pull the total amount from the client into the contract
        client.transfer(&sender, &env.current_contract_address(), &amount);

        // Distribute the funds to all recipients
        for share in shares.iter() {
            let payout = (amount * (share.basis_points as i128)) / 10000;
            if payout > 0 {
                client.transfer(&env.current_contract_address(), &share.recipient, &payout);
            }
        }
    }
}
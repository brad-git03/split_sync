# SplitSync

An automated, trustless payment routing dApp built on Soroban for ad-hoc freelance collectives.

## Problem & Solution
When independent freelancers team up for a single gig, dividing the client's lump-sum payment requires routing it through one person's bank account. This triggers unfair tax liabilities and trust issues. SplitSync solves this by allowing teams to deploy a temporary smart contract that acts as a single payment address. When the client pays, the contract immediately and automatically routes fractional USDC payments to all team members based on pre-set percentages.

## Timeline
* **Day 1:** Soroban contract logic (Basis point validation and multi-transfer routing).
* **Day 2:** Next.js frontend development featuring a minimal, modern SaaS aesthetic (dark green branding, Stripe-like receipt UI).
* **Day 3:** Testnet deployment, Freighter wallet integration, and final demo recording.

## Stellar Features Used
* Soroban Smart Contracts (Rust)
* USDC Transfers (Stable, low-fee settlements)

## Vision and Purpose
To provide decentralized "accounting as a service" for the gig economy, allowing temporary collectives to collaborate and get paid with zero intermediary risk. 

## Prerequisites
* Rust (`rustc 1.74.0+`)
* WebAssembly target (`rustup target add wasm32-unknown-unknown`)
* Soroban CLI (`soroban-cli 20.0.0+`)

## How to Build
Compile the smart contract into a `.wasm` file:
```bash
soroban contract build
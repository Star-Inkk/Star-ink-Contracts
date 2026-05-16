# Deployment Guide

> TODO: expand with full testnet and mainnet deployment steps.

## Prerequisites

- Rust + `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)
- Funded deployer account on target network

## Steps

```bash
# 1. Build
cargo build --target wasm32-unknown-unknown --release

# 2. Optimise
stellar contract optimize \
  --wasm target/wasm32-unknown-unknown/release/star_ink.wasm

# 3. Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/star_ink.optimized.wasm \
  --source deployer \
  --network testnet

# 4. Initialise (replace placeholders)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_ADDRESS> \
  --platform_fee_bps 250
```

## Environment Variables

Copy `.env.example` to `.env` and fill in:

```
CONTRACT_ID=
TOKEN_ADDRESS=
NETWORK=testnet
RPC_URL=https://soroban-testnet.stellar.org
```

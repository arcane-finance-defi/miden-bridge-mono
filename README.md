# Miden Privacy-preserving Crosschain Interoperability Solution

A bridge solution for Miden network, designed to facilitate private token transfers between EVM, and Miden accounts. This bridge serves as an interim solution until the official AggLayer Unified Bridge is launched.

## Overview

The Miden Bridge enables users to:
- Bridge native and ERC-20 tokens from Ethereum Sepolia testnet to Miden accounts
- Bridge native and ERC-20 tokens from Polygon PoS Amoy testnet to Miden accounts
- Bridge assets from Miden accounts back to Ethereum Sepolia or Polygon PoS Amoy testnets
- Maintain privacy during cross-chain transfers through Miden's zero-knowledge technology

## Features

- Privacy-preserving cross-chain token transfers between Ethereum Sepolia and Miden
- Privacy-preserving cross-chain token transfers between Polygon PoS Amoy and Miden
- Support for native tokens and ERC-20 tokens
- CLI interface for bridging assets from Miden to other networks
- Integration with Miden wallet for viewing bridged assets
- Private cross-chain communication leveraging Miden's zero-knowledge technology

## Prerequisites

- Node.js (version X.X.X or higher)
- npm or yarn
- Access to Ethereum Sepolia testnet
- Access to Polygon PoS Amoy testnet
- Miden wallet

## Installation

```bash
# Clone the repository
git clone [repository-url]

# Navigate to the project directory
cd miden-bridge

# Install dependencies
npm install
```

## Usage

### Bridging to Miden

1. Connect your Ethereum or Polygon wallet
2. Select the source network (Ethereum Sepolia or Polygon PoS Amoy)
3. Choose the token to bridge (native or ERC-20)
4. Enter the amount
5. Confirm the transaction
6. View your bridged assets in your Miden wallet

### Bridging from Miden

```bash
# Use the CLI to bridge assets from Miden to other networks
miden-bridge transfer --network [ethereum-sepolia|polygon-amoy] --token [native|erc20] --amount [amount]
```

## Development Status

This bridge is implemented as a simplified solution with the following characteristics:
- Matches AggLayer's API for future integration
- Focuses on functionality over security (as it's a temporary solution)
- Provides basic cross-chain transfer capabilities
- Enables private cross-chain communication between Miden and EVM chains

## Deployments
Currently the bridge support the miden testnet with id 9966 and Sepolia testnet with id 11155111

Sepolia contracts:
- MidenBridgeExtension `0x82a888861cd58e18c474c1d3daf8acc502e5e6ea`
- PolygonBridgeMockProxy `0x77e1099dcc34e82377605a06a6eaa1f68fadc7a5`

The client is [the modificated Miden CLI tool](https://github.com/arcane-finance-defi/miden-bridge-cli) that supports crosschain interactions. You should install it with 

```cargo install --git https://github.com/arcane-finance-defi/miden-bridge-cli miden-cli```
command

## How to use

### EVM to Miden

1. Init miden cli with `miden init` command
2. Generate the wallet with `miden new-wallet`
3. Generate the recipient to the wallet address `miden recipient -a <YOYR ADDRESS>`. Remember the serial number
4. Approve your ERC20 token for the `MidenBridgeExtension` contract
5. Execute [bridgeAndCall](https://github.com/arcane-finance-defi/miden-bridge-evm/blob/488339116ac24b389e48d08d6967dcaffb06db8e/src/MidenBridgeExtension.sol#L39) method of the `MidenBridgeExtension` contract. Use the recipient as the calldata. Set the destination chain param to miden id `9966` and set all addreses to zero (0x0000000000000000000000000000000000000000)
6. Find the miden token wrapper address for your initial token (ask the bridge team)
7. Reconstruct the resulting note with `miden reconstruct --serial-number <SERIAL NUMBER from step 3> --account-id <YOUR ADDRESS from step 2> --asset-amount <AMOUNT from step 5> --faucet-id <FAUCET ID from step 6>` it will reconstruct P2ID note in your storage
8. Call `miden sync`
9. Consume the reconstructed note as usual

### Miden to evm

1. Init miden cli with `miden init` command. Create or import your wallet account with asset in vault
2. Create the crosschain note with `miden crosschain -c <DEST CHAIN ID> -a <DEST ADDRESS> -f <FAUCET ID> -m <AMOUNT> -s <YOUR WALLET ADDRESS>`
3. Import the faucet account (take from public or ask the bridge team for file)
4. Call `miden sync`
5. Consume the crosschain note as usual against the faucet account
6. Wait for the offchain service execution
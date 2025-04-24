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

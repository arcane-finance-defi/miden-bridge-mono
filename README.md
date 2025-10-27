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

## Development Status

This bridge is implemented as a simplified solution with the following characteristics:

- Matches AggLayer's API for future integration
- Focuses on functionality over security (as it's a temporary solution)
- Provides basic cross-chain transfer capabilities
- Enables private cross-chain communication between Miden and EVM chains

## Deployments

Currently the bridge supports Miden testnet (chain_id=9966) and Sepolia testnet (chain_id=11155111)

Sepolia contracts:

- MidenBridgeExtension `0x0b03df1D4B3884b8987254D0C990342B571183AF`
- PolygonBridgeMockProxy `0x341de46031CDBaD4872256F402481B51C2802C5E`

Amoy Contracts:

- MidenBridgeExtension `0x82a888861cd58e18c474c1D3DAf8aCc502e5e6eA`
- PolygonBridgeMockProxy `0x77e1099dcc34E82377605A06a6EaA1f68faDc7A5`

## Supported assets

- USDC Sepolia ([0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238](https://sepolia.etherscan.io/token/0x1c7d4b196cb0c7b01d743fbc6116a902379c7238))/USDC Amoy ([0xdEE54cB0c0D153010b37E6D51AaaD6d682247E5e](https://amoy.polygonscan.com/token/0xdee54cb0c0d153010b37e6d51aaad6d682247e5e)) <=> USDC Miden ([0x4de3bc8d67731a2067af0fcc7a2e34](https://testnet.midenscan.com/account/0x4de3bc8d67731a2067af0fcc7a2e34))

## How to use

The client to interact with the bridge is [the modified Miden CLI tool](https://github.com/arcane-finance-defi/miden-bridge-cli) that supports crosschain interactions. You should install it with

`cargo install --git https://github.com/arcane-finance-defi/miden-bridge-cli miden-client-cli`
command

## Prerequisites

- Node.js (version X.X.X or higher)
- Rust ^1.88.0
- npm or yarn
- Access to Ethereum Sepolia testnet and an address with some gas (you can get it from faucet)
- Access to Polygon PoS Amoy testnet
- Foundry ^1.1.0 (https://getfoundry.sh/)

### EVM to Miden example

1. Init miden cli with `miden-bridge init` command
2. Generate the wallet with `miden-bridge new-wallet`. It will print "Setting account <YOUR MIDEN ADDRESS> as the default account ID." to the console, remember your address.
3. Generate the recipient to the wallet address `miden-bridge recipient -a <YOUR MIDEN ADDRESS>`. Both Recepient and Serial number will be printed to the console, remember them. Note that you can only bridge to the given recepient once.
4. Approve Sepolia USDC for the `MidenBridgeExtension` contract on Sepolia, you can get some on Sepolia Uniswap. The approval can be done in any convenient way, we suggest using a Foundry tool `cast`.
   `cast publish -r https://ethereum-sepolia-rpc.publicnode.com "$(cast mktx -r https://ethereum-sepolia-rpc.publicnode.com --private-key <YOUR EVM PRIVATE KEY> -f <YOUR EVM ADDRESS> 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238 "approve(address,uint256)" 0x0b03df1D4B3884b8987254D0C990342B571183AF <AMOUNT>)"`
5. Execute [bridgeAndCall](https://github.com/arcane-finance-defi/miden-bridge-evm/blob/488339116ac24b389e48d08d6967dcaffb06db8e/src/MidenBridgeExtension.sol#L39) method of the `MidenBridgeExtension` contract. Use the recipient as the calldata. Set the destination chain param to miden id `9966` and set all addreses to zero (0x0000000000000000000000000000000000000000)
   `cast publish -r https://ethereum-sepolia-rpc.publicnode.com "$(cast mktx -r https://ethereum-sepolia-rpc.publicnode.com --private-key <YOUR EVM PRIVATE KEY> -f <YOUR EVM ADDRESS> 0x0b03df1D4B3884b8987254D0C990342B571183AF "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238 <AMOUNT> 9966 0x0000000000000000000000000000000000000000 0x0000000000000000000000000000000000000000 <YOUR RECIPIENT> false)"`
6. Find Miden token wrapper address for your EVM token (for Sepolia USDC it's 0x4de3bc8d67731a2067af0fcc7a2e34), this wrapper address is also a FAUCET_ID for the next step.
7. Call `miden-bridge sync`
8. Reconstruct the resulting note with `miden-bridge reconstruct --serial-number <SERIAL NUMBER from step 3> --account-id <YOUR ADDRESS from step 2> --asset-amount <AMOUNT from step 5> --faucet-id <FAUCET ID from step 6>`. It will reconstruct P2ID note in your storage and print a note id, remember it for step 10.
9. Call `miden-bridge sync`
10. Consume the reconstructed note as usual `miden-bridge consume-notes -a <YOUR MIDEN ADDRESS> <YOUR NOTE ID>`

### Miden to EVM

1. Init miden cli with `miden-bridge init` command. Create or import your wallet account with the asset in the vault (for example, the address from EVM -> Miden bridging)
2. Create the crosschain note with `miden-bridge crosschain -c <DEST CHAIN ID> -a <DEST ADDRESS> -f <FAUCET ID> -m <AMOUNT> -s <MIDEN WALLET ADDRESS>` (Sepolia id is 11155111)
3. Import the faucet account `miden-bridge import-public <FAUCET ADDRESS>`
4. Call `miden-bridge sync`
5. Consume the crosschain note as usual against the faucet account `miden-bridge consume-notes -a <FAUCET ADDRESS> <YOUR NOTE ID from step 2>`
6. Wait for the offchain service execution, the balance should update in your EVM wallet

### Mixer

1. Init miden cli with `miden-bridge init` command. Create or import your wallet account with the asset in the vault (for example, the address from EVM -> Miden bridging)
2. Generate the recipient to the wallet address `miden-bridge recipient --note-type crosschain --dest-chain 11155111 --dest-address <RECEIVER EVM ADDRESS>`. Recepient, BRIDGE Serial number, and Serial number will be printed to the console, remember them.
3. Approve Sepolia USDC for the `MidenBridgeExtension` contract on Sepolia, you can get some on Sepolia Uniswap. The approval can be done in any convenient way, we suggest using a Foundry tool `cast`.
   `cast publish -r https://ethereum-sepolia-rpc.publicnode.com "$(cast mktx -r https://ethereum-sepolia-rpc.publicnode.com --private-key <YOUR PRIVATE KEY> -f <YOUR ADDRESS> 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238 "approve(address,uint256)" 0x82a888861cd58e18c474c1d3daf8acc502e5e6ea <AMOUNT>)"`
4. Execute [bridgeAndCall](https://github.com/arcane-finance-defi/miden-bridge-evm/blob/488339116ac24b389e48d08d6967dcaffb06db8e/src/MidenBridgeExtension.sol#L39) method of the `MidenBridgeExtension` contract. Use the recipient as the calldata. Set the destination chain param to miden id `9966` and set all addreses to zero (0x0000000000000000000000000000000000000000)
   `cast publish -r https://ethereum-sepolia-rpc.publicnode.com "$(cast mktx -r https://ethereum-sepolia-rpc.publicnode.com --private-key <YOUR PRIVATE KEY> -f <YOUR ADDRESS> 0x82a888861cd58e18c474c1d3daf8acc502e5e6ea "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238 <AMOUNT> 9966 0x0000000000000000000000000000000000000000 0x0000000000000000000000000000000000000000 <YOUR RECIPIENT> false)"`
5. Provide the intermediate note to the mixer offchain service `miden-bridge mix --serial-number <SERIAL NUMBER> --bridge-serial-number <BRIDGE SERIAL NUMBER> --dest-chain 11155111 --dest-address <RECEIVER EVM ADDRESS> --faucet-id 0xd354f13600df2920000c682da84a64 --asset-amount <AMOUNT>`
6. Wait for the offchain service execution, the balance should update in your EVM wallet

# Script

You could use the script for the testing purposes.

`bash -c ./bridge.sh`

Script supports the bridging and mixing features.

## Bridging

On script start select 1 or 2 option for the bridge usage. The script collects the required data interactively

## Mixing

Ensure you installed the latest miden-bridge CLI revision. Select option 3 in script scenario

# Developers

## Installation

```bash
# Clone the repository
git clone [repository-url]

# Navigate to the project directory
cd miden-bridge-mono

# Install dependencies
cd evm
npm install

cd ../relayer/api
npm install
npm run gen

cd ../evm-side
npm install

cd ../miden-tx-sender
cargo fetch
```

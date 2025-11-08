export const EVM_RPCS = 'EVM_RPCS';
export const MIDEN_RPCS = 'MIDEN_RPCS';
export const SOLANA_RPCS = 'SOLANA_RPCS';

export const EVM_BRIDGE_ADDRESSES = 'EVM_BRIDGE_ADDRESSES';
export const SOLANA_BRIDGE_ADDRESSES = 'SOLANA_BRIDGE_ADDRESSES';

export function getEvmStartBlockScanEnvVarKey(
  chainId: bigint,
): `EVM_START_SCAN_BLOCK_${string}` {
  return `EVM_START_SCAN_BLOCK_${chainId.toString()}`;
}

export function getEvmFinalizationBlockGapEnvVarKey(
  chainId: bigint,
): `EVM_FINALIZATION_BLOCK_GAP_${string}` {
  return `EVM_FINALIZATION_BLOCK_GAP_${chainId.toString()}`;
}

export function getEvmScanBatchSizeEnvVarKey(
  chainId: bigint,
): `EVM_SCAN_BATCH_SIZE_${string}` {
  return `EVM_SCAN_BATCH_SIZE_${chainId.toString()}`;
}

export function getEvmWithdrawContractEnvVarKey(
  chainId: bigint,
): `EVM_WITHDRAW_CONTRACT_ADDRESS_${string}` {
  return `EVM_WITHDRAW_CONTRACT_ADDRESS_${chainId.toString()}`;
}

export function getEvmWithdrawContractOwnerPkEnvVarKey(
  chainId: bigint,
): `EVM_WITHDRAW_CONTRACT_OWNER_PK_${string}` {
  return `EVM_WITHDRAW_CONTRACT_OWNER_PK_${chainId.toString()}`;
}

export function getMidenStartBlockScanEnvVarKey(
  chainId: bigint,
): `MIDEN_START_SCAN_BLOCK_${string}` {
  return `MIDEN_START_SCAN_BLOCK_${chainId.toString()}`;
}

export function getSolanaStartBlockScanEnvVarKey(
  chainId: bigint,
): `SOLANA_START_SCAN_BLOCK_${string}` {
  return `SOLANA_START_SCAN_BLOCK_${chainId.toString()}`;
}

export function getSolanaBridgeAddressEnvVarKey(
  chainId: bigint,
): `SOLANA_BRIDGE_ADDRESS_${string}` {
  return `SOLANA_BRIDGE_ADDRESS_${chainId.toString()}`;
}

export function getSolanaBridgeUserPrivateKeyEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_BRIDGE_USER_PRIVATE_KEY_${string}` {
  return `SOLANA_BRIDGE_USER_PRIVATE_KEY_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

export function getSolanaAssetOriginNetworkEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_ASSET_ORIGIN_NETWORK_${string}` {
  return `SOLANA_ASSET_ORIGIN_NETWORK_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

export function getSolanaAssetOriginAddressEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_ASSET_ORIGIN_ADDRESS_${string}` {
  return `SOLANA_ASSET_ORIGIN_ADDRESS_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

export function getSolanaAssetDestinationAddressEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_ASSET_DESTINATION_ADDRESS_${string}` {
  return `SOLANA_ASSET_DESTINATION_ADDRESS_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

export function getSolanaAssetDecimalsEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_ASSET_DECIMALS_${string}` {
  return `SOLANA_ASSET_DECIMALS_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

export function getSolanaAssetSymbolEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_ASSET_SYMBOL_${string}` {
  return `SOLANA_ASSET_SYMBOL_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

export function getSolanaAssetNameEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_ASSET_NAME_${string}` {
  return `SOLANA_ASSET_NAME_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

export function getSolanaBridgeFirstSignatureEnvVarKey(
  chainId: bigint,
  bridgeAddress: string,
): `SOLANA_BRIDGE_FIRST_SIGNATURE_${string}` {
  return `SOLANA_BRIDGE_FIRST_SIGNATURE_${chainId.toString()}_${bridgeAddress.toUpperCase()}`;
}

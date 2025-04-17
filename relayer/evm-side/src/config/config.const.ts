export const EVM_RPCS = 'EVM_RPCS';
export const MIDEN_RPCS = 'MIDEN_RPCS';

export const EVM_BRIDGE_ADDRESSES = 'EVM_BRIDGE_ADDRESSES';

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

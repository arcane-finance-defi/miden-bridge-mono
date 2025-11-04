import { ConfigService } from '@nestjs/config';
import { isAddress } from 'ethers';
import { isAddress as isSolanaAddress } from '@solana/kit';

export class BridgeAddressService {
  static getBridgeAddress(
    evmChainIds: Array<bigint>,
    config: ConfigService,
  ): Map<bigint, string> {
    const result: Map<bigint, string> = new Map();
    for (const chainId of evmChainIds) {
      const envKey = `EVM_BRIDGE_ADDRESS_CHAIN_${chainId}`;
      const address: string = config.getOrThrow(envKey);
      if (!isAddress(address)) {
        throw new Error(`Malformed address in env var "${envKey}"`);
      }

      result.set(chainId, address);
    }

    return result;
  }

  static getSolanaBridgeAddress(
    solanaChainIds: Array<bigint>,
    config: ConfigService,
  ): Map<bigint, string> {
    const result: Map<bigint, string> = new Map();
    for (const chainId of solanaChainIds) {
      const envKey = `SOLANA_BRIDGE_ADDRESS_${chainId}`;
      const address: string = config.getOrThrow(envKey);
      if (!isSolanaAddress(address)) {
        throw new Error(`Malformed address in env var "${envKey}"`);
      }

      result.set(chainId, address);
    }
    return result;
  }
}

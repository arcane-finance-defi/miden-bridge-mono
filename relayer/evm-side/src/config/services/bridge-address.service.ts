import { ConfigService } from '@nestjs/config';
import { isAddress } from 'ethers';

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
}

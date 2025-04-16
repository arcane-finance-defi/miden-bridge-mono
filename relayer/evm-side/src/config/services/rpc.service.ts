import { JsonRpcProvider, Provider } from 'ethers';
import { ConfigService } from '@nestjs/config';
import { MidenApiService } from './miden.service';

export class RpcConfigService {
  static async connectEvmChains(
    evmChainIds: Array<bigint>,
    config: ConfigService,
  ): Promise<Map<bigint, Provider>> {
    const result: Map<bigint, Provider> = new Map();
    for (const chainId of evmChainIds) {
      const envKey = `EVM_RPC_CHAIN_${chainId}`;
      const url: string = config.getOrThrow(envKey);
      if (!URL.canParse(url)) {
        throw new Error(`Malformed url in env var "${envKey}"`);
      }

      const provider = new JsonRpcProvider(url);
      const { chainId: providedChainId } = await provider.getNetwork();
      if (chainId !== providedChainId) {
        throw new Error(
          `Unexpected chain id "${providedChainId}" from rpc "${envKey}"`,
        );
      }

      result.set(chainId, provider);
    }

    return result;
  }

  static async connectMidenChains(
    midenChainIds: Array<bigint>,
    config: ConfigService,
  ): Promise<Map<bigint, MidenApiService>> {
    const result: Map<bigint, MidenApiService> = new Map();

    for (const chainId of midenChainIds) {
      const envKey = `MIDEN_RPC_CHAIN_${chainId}`;
      const url: string = config.getOrThrow(envKey);
      if (!URL.canParse(url)) {
        throw new Error(`Malformed url in env var "${envKey}"`);
      }

      const provider = new MidenApiService(url);

      result.set(chainId, provider);
    }
    return result;
  }
}

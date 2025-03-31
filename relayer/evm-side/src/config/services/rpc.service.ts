import { JsonRpcProvider, Provider } from 'ethers';
import { ConfigService } from '@nestjs/config';

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
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _midenChainIds: Array<bigint>,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _config: ConfigService,
  ): Promise<Map<bigint, any>> {
    const result: Map<bigint, Provider> = new Map();
    return result;
  }
}

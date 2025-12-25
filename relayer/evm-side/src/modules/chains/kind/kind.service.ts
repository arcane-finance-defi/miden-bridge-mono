import { Injectable } from '@nestjs/common';
import { MainConfigService } from 'src/config';
import { ChainRef } from 'src/models/exit.model';

@Injectable()
export class KindService {
  private readonly midenChains: Array<bigint>;
  private readonly solanaChains: Array<bigint>;

  constructor(config: MainConfigService) {
    this.midenChains = config.getMidenChainIds();
    this.solanaChains = config.getSolanaChainIds();
  }

  getChainKind(chainId: bigint | number): ChainRef['chainKind'] {
    const chainIdBigInt = BigInt(chainId);
    if (this.midenChains.includes(chainIdBigInt)) {
      return 'miden';
    } else if (this.solanaChains.includes(chainIdBigInt)) {
      return 'solana';
    }
    return 'evm';
  }
}

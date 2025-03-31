import { WINSTON_MODULE_PROVIDER } from 'nest-winston';
import { Inject, Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { Logger } from 'winston';

import { IEnvConfig } from '../interfaces';

function parseNumberList(value: string): Array<bigint> {
  return value.split(',').map((v) => BigInt(v));
}

@Injectable()
export class MainConfigService {
  private readonly evmChainIds: Array<bigint>;
  private readonly midenChainIds: Array<bigint>;

  constructor(
    @Inject(WINSTON_MODULE_PROVIDER)
    private readonly logger: Logger,
    private readonly configService: ConfigService<IEnvConfig>,
  ) {
    this.evmChainIds = parseNumberList(
      configService.getOrThrow('EVM_CHAIN_IDS'),
    );
    this.midenChainIds = parseNumberList(
      configService.getOrThrow('MIDEN_CHAIN_IDS'),
    );
  }

  getEnv<K extends keyof IEnvConfig>(key: K): IEnvConfig[K] {
    return this.configService.get(key)!;
  }

  getEvmChainConf<K extends `EVM_${string}`>(key: K): string {
    return process.env[key];
  }

  getEvmChainIds(): Array<bigint> {
    return this.evmChainIds;
  }

  getMidenChainIds(): Array<bigint> {
    return this.midenChainIds;
  }
}

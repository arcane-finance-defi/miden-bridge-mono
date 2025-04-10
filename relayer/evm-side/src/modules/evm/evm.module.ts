import {
  ConfigurableModuleAsyncOptions,
  DynamicModule,
  FactoryProvider,
  Module,
} from '@nestjs/common';
import { ConfigurableModuleClass } from './evm.module-definition';
import { EvmModuleOptions } from './interfaces/evm-module-options.interface';
import { PollerService } from './services/poller.service';
import { MainConfigModule, MainConfigService } from 'src/config';
import { RpcService } from './services/rpc.service';
import { RepositoriesModule } from 'src/repositories/repositories.module';
import { ExitRepository } from 'src/repositories/services/exit.repository';
import { ScansRepository } from 'src/repositories/services/scans.repository';
import { MidenModule } from '../miden';

function generateKey(chainId): string {
  return `evm-poller-${chainId}`;
}

function generateProvider(chainId): FactoryProvider<PollerService> {
  return {
    provide: generateKey(chainId),
    useFactory(config, rpc, scans, exits) {
      return new PollerService(rpc, chainId, config, scans, exits);
    },
    inject: [MainConfigService, RpcService, ScansRepository, ExitRepository],
  };
}

function generateAsyncProvider(index): FactoryProvider<PollerService> {
  return {
    provide: generateKey(index),
    useFactory(config: MainConfigService, rpc, scans, exits) {
      const chainId = config.getEvmChainIds()[index];
      return new PollerService(rpc, chainId, config, scans, exits);
    },
    inject: [MainConfigService, RpcService, ScansRepository, ExitRepository],
  };
}

@Module({
  imports: [MainConfigModule, RepositoriesModule, MidenModule],
  providers: [RpcService],
})
export class EvmModule extends ConfigurableModuleClass {
  static register({ chainIds }: EvmModuleOptions): DynamicModule {
    const pollers = chainIds.map((chainId) => generateProvider(chainId));

    const module = super.register({
      chainIds,
    });

    return {
      module: EvmModule,
      ...module,
      providers: [...pollers, ...module.providers],
    };
  }

  static registerAsync(
    options: ConfigurableModuleAsyncOptions<EvmModuleOptions>,
  ) {
    const module = super.registerAsync(options);

    const evmRpcsCount = process.env.EVM_CHAIN_IDS!.split(',').length;
    const pollerProviders = new Array(evmRpcsCount)
      .fill(0)
      .map((_v, index) => generateAsyncProvider(index));

    return {
      module: EvmModule,
      ...module,
      providers: [...pollerProviders, ...module.providers],
    };
  }
}

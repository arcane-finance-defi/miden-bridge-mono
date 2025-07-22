import {
  ConfigurableModuleAsyncOptions,
  DynamicModule,
  FactoryProvider,
  Module,
} from '@nestjs/common';
import { ConfigurableModuleClass } from './miden.module-definition';
import { MidenModuleOptions } from './interfaces/miden-module-options.interface';
import { PollerService } from './services/poller.service';
import {
  MainConfigModule,
  MainConfigService,
  MIDEN_RPCS,
  MidenApiService,
} from 'src/config';
import { RepositoriesModule } from 'src/repositories/repositories.module';
import { ExitRepository } from 'src/repositories/services/exit.repository';
import { ScansRepository } from 'src/repositories/services/scans.repository';

function generateKey(chainId): string {
  return `miden-poller-${chainId}`;
}

function generateProvider(chainId): FactoryProvider<PollerService> {
  return {
    provide: generateKey(chainId),
    useFactory(rpcs: Map<bigint, MidenApiService>, exits, scans, config) {
      const rpc = rpcs.get(chainId);
      if (rpc == null) {
        throw new Error(`Unknown miden chain with chainId: ${chainId}`);
      }
      return new PollerService(chainId, rpc, exits, scans, config);
    },
    inject: [MIDEN_RPCS, ExitRepository, ScansRepository, MainConfigService],
  };
}

function generateAsyncProvider(index): FactoryProvider<PollerService> {
  return {
    provide: generateKey(index),
    useFactory(
      config: MainConfigService,
      rpcs: Map<bigint, MidenApiService>,
      exits,
      scans,
    ) {
      const chainId = config.getMidenChainIds()[index];
      const rpc = rpcs.get(chainId);
      if (rpc == null) {
        throw new Error(`Unknown miden chain with chainId: ${chainId}`);
      }
      return new PollerService(chainId, rpc, exits, scans, config);
    },
    inject: [MainConfigService, MIDEN_RPCS, ExitRepository, ScansRepository],
  };
}

@Module({
  imports: [MainConfigModule, RepositoriesModule],
})
export class MidenModule extends ConfigurableModuleClass {
  static register({ chainIds }: MidenModuleOptions): DynamicModule {
    const pollers = chainIds.map((chainId) => generateProvider(chainId));

    const module = super.register({
      chainIds,
    });

    return {
      module: MidenModule,
      ...module,
      providers: [...pollers, ...module.providers],
    };
  }

  static registerAsync(
    options: ConfigurableModuleAsyncOptions<MidenModuleOptions>,
  ) {
    const module = super.registerAsync(options);

    const evmRpcsCount = process.env.MIDEN_CHAIN_IDS!.split(',').length;
    const pollerProviders = new Array(evmRpcsCount)
      .fill(0)
      .map((_v, index) => generateAsyncProvider(index));

    return {
      module: MidenModule,
      ...module,
      providers: [...pollerProviders, ...module.providers],
    };
  }
}

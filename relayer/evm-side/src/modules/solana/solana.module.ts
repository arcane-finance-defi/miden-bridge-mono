import {
  ConfigurableModuleAsyncOptions,
  DynamicModule,
  FactoryProvider,
  Module,
} from '@nestjs/common';
import { ConfigurableModuleClass } from './solana.module-definition';
import { SolanaModuleOptions } from './interfaces/solana-module-options.interface';
import { PollerService } from './services/poller/poller.service';
import { MainConfigModule, MainConfigService, SOLANA_RPCS } from 'src/config';
import { RepositoriesModule } from 'src/repositories/repositories.module';
import { ExitRepository } from 'src/repositories/services/exit.repository';
import { ScansRepository } from 'src/repositories/services/scans.repository';
import { KindService } from '../chains/kind/kind.service';
import { ChainsModule } from '../chains/chains.module';
import { SolscansService } from 'src/repositories/services/solscans.repository';
import { Rpc, SolanaRpcApi } from '@solana/kit';

function generateKey(chainId): string {
  return `solana-poller-${chainId}`;
}

function generateProvider(chainId): FactoryProvider<PollerService> {
  return {
    provide: generateKey(chainId),
    useFactory(
      rpcs: Map<bigint, Rpc<SolanaRpcApi>>,
      scans: ScansRepository,
      solscans: SolscansService,
      exits: ExitRepository,
      config: MainConfigService,
      kindService: KindService,
    ) {
      const rpc = rpcs.get(chainId);
      if (rpc == null) {
        throw new Error(`Unknown solana chain with chainId: ${chainId}`);
      }
      return new PollerService(
        chainId,
        scans,
        solscans,
        exits,
        config,
        rpc,
        kindService,
      );
    },
    inject: [
      SOLANA_RPCS,
      ScansRepository,
      SolscansService,
      ExitRepository,
      MainConfigService,
      KindService,
    ],
  };
}

function generateAsyncProvider(index): FactoryProvider<PollerService> {
  return {
    provide: generateKey(index),
    useFactory(
      rpcs: Map<bigint, Rpc<SolanaRpcApi>>,
      scans: ScansRepository,
      solscans: SolscansService,
      exits: ExitRepository,
      config: MainConfigService,
      kindService: KindService,
    ) {
      const chainId = config.getSolanaChainIds()[index];
      const rpc = rpcs.get(chainId);
      if (rpc == null) {
        throw new Error(`Unknown solana chain with chainId: ${chainId}`);
      }
      return new PollerService(
        chainId,
        scans,
        solscans,
        exits,
        config,
        rpc,
        kindService,
      );
    },
    inject: [
      SOLANA_RPCS,
      ScansRepository,
      SolscansService,
      ExitRepository,
      MainConfigService,
      KindService,
    ],
  };
}

@Module({
  imports: [MainConfigModule, RepositoriesModule, ChainsModule],
})
export class SolanaModule extends ConfigurableModuleClass {
  static register({ chainIds }: SolanaModuleOptions): DynamicModule {
    const pollers = chainIds.map((chainId) => generateProvider(chainId));

    const module = super.register({
      chainIds,
    });

    return {
      module: SolanaModule,
      ...module,
      providers: [...pollers, ...module.providers],
    };
  }

  static registerAsync(
    options: ConfigurableModuleAsyncOptions<SolanaModuleOptions>,
  ) {
    const module = super.registerAsync(options);

    const evmRpcsCount = process.env.SOLANA_CHAIN_IDS!.split(',').length;
    const pollerProviders = new Array(evmRpcsCount)
      .fill(0)
      .map((_v, index) => generateAsyncProvider(index));

    return {
      module: SolanaModule,
      ...module,
      providers: [...pollerProviders, ...module.providers],
    };
  }
}

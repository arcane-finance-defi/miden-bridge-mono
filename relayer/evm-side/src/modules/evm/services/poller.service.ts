import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { RpcService } from './rpc.service';
import { ScansRepository } from 'src/repositories/services/scans.repository';
import { ExitRepository } from 'src/repositories/services/exit.repository';
import {
  getEvmFinalizationBlockGapEnvVarKey,
  getEvmScanBatchSizeEnvVarKey,
  getEvmStartBlockScanEnvVarKey,
  MainConfigService,
} from 'src/config';
import { ChainRef, ExitModel } from 'src/models/exit.model';
import BigNumber from 'bignumber.js';

@Injectable()
export class PollerService {
  private readonly logger: Logger;
  private readonly startScanBlock: number;
  private readonly finalizationBlockGap: number;
  private readonly scanBatchSize: number;

  private readonly midenChains: Array<bigint>;
  private readonly evmChains: Array<bigint>;

  constructor(
    private readonly rpc: RpcService,
    private readonly chainId: bigint,
    config: MainConfigService,
    private readonly scans: ScansRepository,
    private readonly exits: ExitRepository,
  ) {
    this.logger = new Logger(`EvmPollerService (${chainId})`);
    this.startScanBlock = Number.parseInt(
      config.getEvmChainConf(getEvmStartBlockScanEnvVarKey(chainId)) || '0',
    );
    this.finalizationBlockGap = Number.parseInt(
      config.getEvmChainConf(getEvmFinalizationBlockGapEnvVarKey(chainId)) ||
        '0',
    );
    this.scanBatchSize = Number.parseInt(
      config.getEvmChainConf(getEvmScanBatchSizeEnvVarKey(chainId)) || '100',
    );

    this.evmChains = config.getEvmChainIds();
    this.midenChains = config.getMidenChainIds();
  }

  private getChainKind(chainId: bigint): ChainRef['chainKind'] {
    if (this.midenChains.includes(chainId)) {
      return 'miden';
    }
    return 'evm';
  }

  @Cron(CronExpression.EVERY_10_SECONDS, { waitForCompletion: true })
  async poll() {
    const rpcHeight = await this.rpc.getChainHeight(this.chainId);
    const lastScannedHeight = await this.scans.getLastScannedBlockFor(
      this.chainId,
    );

    const startBlock = Math.max(lastScannedHeight + 1, this.startScanBlock);
    const endBlock = Math.min(
      startBlock + this.scanBatchSize,
      rpcHeight - this.finalizationBlockGap,
    );

    if (endBlock <= startBlock) {
      return; //skip
    }

    const events = await this.rpc.getBridgeEvents(
      this.chainId,
      startBlock,
      endBlock,
    );

    const exits: Array<Partial<ExitModel>> = events.map(([asset, message]) => ({
      txId: asset.tx,
      assetAddress: message.metadata.assetOriginalAddr,
      assetOrigin: {
        chainId: message.metadata.assetOriginalNetwork,
        chainKind: this.getChainKind(message.metadata.assetOriginalNetwork),
      },
      assetAmount: new BigNumber(asset.amount.toString()),
      from: {
        chainId: asset.originNetwork,
        chainKind: this.getChainKind(asset.originNetwork),
      },
      to: {
        chainId: asset.destinationNetwork,
        chainKind: this.getChainKind(asset.destinationNetwork),
      },
      sender: asset.originAddress,
      receiver: message.metadata.callAddress,
      assetName: asset?.metadata?.name,
      assetSymbol: asset?.metadata?.symbol,
      assetDecimals: asset?.metadata?.deimals,
    }));

    this.logger.log(`Found ${exits.length} exits from chain`);

    await this.exits.tx(async (em) => {
      for (const exit of exits) {
        await this.exits.insertExit(exit, em);
      }

      await this.scans.insertScan({
        chain: {
          chainId: this.chainId,
          chainKind: 'evm',
        },
        startBlock,
        endBlock,
      });
    });

    this.logger.log(`Scan to block ${endBlock} saved`);
  }
}

import { Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import BigNumber from 'bignumber.js';
import {
  getMidenStartBlockScanEnvVarKey,
  MainConfigService,
  MidenApiService,
} from 'src/config';
import { ChainRef } from 'src/models/exit.model';
import { ExitRepository, ScansRepository } from 'src/repositories';

export class PollerService {
  private readonly logger;
  private readonly startScanBlock: number;

  private readonly midenChains: Array<bigint>;
  private readonly evmChains: Array<bigint>;

  constructor(
    private readonly chainId: bigint,
    private readonly rpc: MidenApiService,
    private readonly exits: ExitRepository,
    private readonly scans: ScansRepository,
    config: MainConfigService,
  ) {
    this.logger = new Logger(`MidenPollerService (${chainId})`);
    this.startScanBlock = Number.parseInt(
      config.getMidenChainConf(getMidenStartBlockScanEnvVarKey(chainId)) || '0',
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
    const lastScannedHeight = await this.scans.getLastScannedBlockFor(
      this.chainId,
    );

    const startBlock = Math.max(lastScannedHeight + 1, this.startScanBlock);

    const { chainTip, events } = await this.rpc.pollExits(startBlock);

    this.logger.log(`Found ${events.length} exits from chain`);
    await this.exits.tx(async (em) => {
      for (const exit of events) {
        await this.exits.insertExit(
          {
            from: {
              chainId: this.chainId,
              chainKind: 'miden',
            },
            to: {
              chainId: BigInt(exit.destinationChain),
              chainKind: this.getChainKind(BigInt(exit.destinationChain)),
            },
            assetAddress: exit.asset.originAddress,
            assetOrigin: {
              chainId: BigInt(exit.asset.originNetwork),
              chainKind: this.getChainKind(BigInt(exit.asset.originNetwork)),
            },
            assetAmount: BigNumber(exit.amount),
            assetDecimals: exit.asset.decimals,
            assetSymbol: exit.asset.assetSymbol,
            blockNumber: exit.blockNumber,
            receiver: exit.receiver,
            calldata: exit.callData,
            callAddress: exit.callAddress,
          },
          em,
        );
      }
      await this.scans.insertScan(
        {
          chain: {
            chainId: this.chainId,
            chainKind: 'miden',
          },
          startBlock,
          endBlock: chainTip,
        },
        em,
      );

      this.logger.log(`Scan to block ${chainTip} saved`);
    });
  }
}

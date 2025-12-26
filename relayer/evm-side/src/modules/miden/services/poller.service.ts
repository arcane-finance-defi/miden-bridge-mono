import { Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { getAddressDecoder } from '@solana/kit';
import BigNumber from 'bignumber.js';
import {
  getMidenStartBlockScanEnvVarKey,
  MainConfigService,
  MidenApiService,
} from 'src/config';
import { KindService } from 'src/modules/chains/kind/kind.service';
import { ExitRepository, ScansRepository } from 'src/repositories';

export class PollerService {
  private readonly logger;
  private readonly startScanBlock: number;

  constructor(
    private readonly chainId: bigint,
    private readonly rpc: MidenApiService,
    private readonly exits: ExitRepository,
    private readonly scans: ScansRepository,
    config: MainConfigService,
    private readonly kindService: KindService,
  ) {
    this.logger = new Logger(`MidenPollerService (${chainId})`);
    this.startScanBlock = Number.parseInt(
      config.getMidenChainConf(getMidenStartBlockScanEnvVarKey(chainId)) || '0',
    );
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
        const destinationChainKind = this.kindService.getChainKind(
          BigInt(exit.destinationChain),
        );

        const receiverBytes = Buffer.from(
          exit.receiver.replace('0x', ''),
          'hex',
        );
        let receiver: string;
        switch (destinationChainKind) {
          case 'solana':
            const decoder = getAddressDecoder();
            receiver = decoder.decode(receiverBytes.subarray(0, 32));
            break;
          case 'evm':
            receiver = '0x' + receiverBytes.subarray(0, 20).toString('hex');
            break;
        }

        await this.exits.insertExit(
          {
            from: {
              chainId: this.chainId,
              chainKind: 'miden',
            },
            to: {
              chainId: BigInt(exit.destinationChain),
              chainKind: destinationChainKind,
            },
            assetAddress: exit.asset.originAddress,
            assetOrigin: {
              chainId: BigInt(exit.asset.originNetwork),
              chainKind: this.kindService.getChainKind(
                BigInt(exit.asset.originNetwork),
              ),
            },
            assetAmount: BigNumber(exit.amount),
            assetDecimals: exit.asset.decimals,
            assetSymbol: exit.asset.assetSymbol,
            blockNumber: exit.blockNumber,
            receiver,
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

import { Injectable } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { ZeroAddress } from 'ethers';
import { MidenApiService } from 'src/modules/miden';
import { ExitRepository } from 'src/repositories/services/exit.repository';

const DEFAULT_RELAY_BATCH_SIZE = 20;

@Injectable()
export class RelayerService {
  private readonly batchSize = DEFAULT_RELAY_BATCH_SIZE;

  constructor(
    private readonly exits: ExitRepository,
    private readonly miden: MidenApiService,
  ) {}

  @Cron(CronExpression.EVERY_MINUTE, { waitForCompletion: true })
  async doRelay() {
    await this.exits.tx(async (em) => {
      const [firstPage, exitsCount] = await this.exits.getPendingExitsPage(
        this.batchSize,
        0,
        em,
      );

      const pagesCount = Math.ceil(exitsCount / this.batchSize);
      for (let [index, page] = [0, firstPage]; index < pagesCount; index++) {
        for (const exit of page) {
          if (exit.to.chainKind === 'miden') {
            const isWethAsset =
              exit.assetOrigin.chainId === 0n &&
              exit.assetAddress === ZeroAddress;
            await this.miden.send({
              amount: exit.assetAmount,
              asset: {
                address: exit.assetAddress,
                decimals: isWethAsset ? 18 : exit.assetDecimals,
                network: Number(exit.assetOrigin.chainId),
                symbol: isWethAsset ? 'WETH' : exit.assetSymbol,
              },
              recipient: exit.calldata,
            });
          } else {
            throw new Error('EVM destination not implemented yet');
          }
        }
        await this.exits.fulfill(
          page.map((item) => item.id),
          em,
        );
        [page] = await this.exits.getPendingExitsPage(this.batchSize, 0, em);
      }
    });
  }
}

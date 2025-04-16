import { Inject, Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { ZeroAddress } from 'ethers';
import { MIDEN_RPCS, MidenApiService } from 'src/config';
import { ExitRepository } from 'src/repositories/services/exit.repository';

const DEFAULT_RELAY_BATCH_SIZE = 20;

@Injectable()
export class RelayerService {
  private readonly batchSize = DEFAULT_RELAY_BATCH_SIZE;
  private readonly logger = new Logger(RelayerService.name);

  constructor(
    private readonly exits: ExitRepository,
    @Inject(MIDEN_RPCS)
    private readonly midenRpcs: Map<bigint, MidenApiService>,
  ) {}

  @Cron(CronExpression.EVERY_MINUTE, { waitForCompletion: true })
  async doRelay() {
    await this.exits.tx(async (em) => {
      this.logger.debug('Relay job initiated');

      const [firstPage, exitsCount] = await this.exits.getPendingExitsPage(
        this.batchSize,
        0,
        em,
      );

      const pagesCount = Math.ceil(exitsCount / this.batchSize);
      this.logger.debug(`Relayer have ${pagesCount} pages to process`);
      for (let [index, page] = [0, firstPage]; index < pagesCount; index++) {
        for (const exit of page) {
          const rpc = this.midenRpcs.get(exit.to.chainId);
          if (rpc == null) {
            throw new Error(
              `Unknown miden chain with chainId: ${exit.to.chainId}`,
            );
          }
          if (exit.to.chainKind === 'miden') {
            const isWethAsset =
              exit.assetOrigin.chainId === 0n &&
              exit.assetAddress === ZeroAddress;
            this.logger.debug(
              `Exit with id ${exit.id} is about to relay to miden chain`,
            );
            const response = await rpc.send({
              amount: exit.assetAmount,
              asset: {
                address: exit.assetAddress,
                decimals: isWethAsset ? 18 : exit.assetDecimals,
                network: Number(exit.assetOrigin.chainId),
                symbol: isWethAsset ? 'WETH' : exit.assetSymbol,
              },
              recipient: exit.calldata,
            });

            this.logger.debug(
              `Deposit relayed to the miden with note id: ${response.noteId}`,
            );
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

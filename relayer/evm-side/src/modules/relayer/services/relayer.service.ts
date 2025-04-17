import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { ExitRepository } from 'src/repositories/services/exit.repository';
import { MidenRelayerService } from './miden.service';
import { EVMRelayerService } from './evm.service';

const DEFAULT_RELAY_BATCH_SIZE = 20;

@Injectable()
export class RelayerService {
  private readonly batchSize = DEFAULT_RELAY_BATCH_SIZE;
  private readonly logger = new Logger(RelayerService.name);

  constructor(
    private readonly exits: ExitRepository,
    private readonly miden: MidenRelayerService,
    private readonly evm: EVMRelayerService,
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
          if (exit.to.chainKind === 'miden') {
            await this.miden.relay(exit);
          } else {
            await this.evm.relay(exit);
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

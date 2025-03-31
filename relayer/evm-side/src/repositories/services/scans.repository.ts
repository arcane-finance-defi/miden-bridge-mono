import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { ScanModel } from 'src/models/scan.model';
import { EntityManager, Equal, Repository } from 'typeorm';

@Injectable()
export class ScansRepository {
  constructor(
    @InjectRepository(ScanModel)
    private readonly inner: Repository<ScanModel>,
  ) {}

  async insertScan(
    model: Partial<ScanModel>,
    manager: EntityManager = this.inner.manager,
  ): Promise<void> {
    await manager.getRepository(ScanModel).insert(model);
  }

  async getLastScannedBlockFor(chainId: bigint): Promise<number> {
    const scan = await this.inner.findOne({
      where: {
        chain: {
          chainId: Equal(chainId),
        },
      },
      order: {
        endBlock: 'DESC',
      },
    });

    return scan?.endBlock || 0;
  }
}

import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { ScanModel } from 'src/models/scan.model';
import {
  MoreThanOrEqual,
  EntityManager,
  Equal,
  LessThan,
  Repository,
  LessThanOrEqual,
  MoreThan,
} from 'typeorm';

@Injectable()
export class ScansRepository {
  constructor(
    @InjectRepository(ScanModel)
    private readonly inner: Repository<ScanModel>,
  ) {}

  async insertScan(
    model: Partial<ScanModel>,
    manager: EntityManager = this.inner.manager,
  ): Promise<string> {
    const result = await manager
      .getRepository(ScanModel)
      .createQueryBuilder()
      .insert()
      .values(model)
      .returning(['id'])
      .execute();

    return result.raw[0].id;
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

  async getScanGapsFor(
    chainId: bigint,
    startBlock: number | bigint,
    endBlock: number | bigint,
  ): Promise<
    Array<{
      startBlock: number;
      endBlock: number;
      latestSignature?: string;
      earliestSignature?: string;
    }>
  > {
    const numericChainId = Number.parseInt(chainId.toString());
    const scans = await this.inner
      .createQueryBuilder('s')
      .where({
        chain: {
          chainId: numericChainId,
        },
      })
      .andWhere([
        {
          startBlock: LessThanOrEqual(startBlock),
          endBlock: MoreThan(startBlock),
        },
        {
          startBlock: MoreThanOrEqual(startBlock),
          endBlock: LessThan(endBlock),
        },
        {
          startBlock: LessThanOrEqual(endBlock),
          endBlock: MoreThan(endBlock),
        },
      ])
      .innerJoinAndSelect('s.solanaScan', 'ss')
      .orderBy('s.startBlock', 'ASC')
      .getMany();

    if (scans.length === 0) {
      return [
        {
          startBlock: Number.parseInt(startBlock.toString()),
          endBlock: Number.parseInt(endBlock.toString()),
          latestSignature: null,
          earliestSignature: null,
        },
      ];
    }

    return scans
      .map(({ endBlock: scanEndBlock }, idx, all) => {
        if (all.length === idx + 1) {
          return {
            startBlock: scanEndBlock,
            endBlock: Number.parseInt(endBlock.toString()),
            latestSignature: all[idx].solanaScan?.latestSignature,
            earliestSignature: all[idx].solanaScan?.earliestSignature,
          };
        }
        return {
          startBlock: scanEndBlock,
          endBlock: all[idx + 1].startBlock,
          latestSignature: all[idx + 1].solanaScan?.latestSignature,
          earliestSignature: all[idx].solanaScan?.earliestSignature,
        };
      })
      .filter(({ startBlock, endBlock }) => endBlock - startBlock > 1);
  }
}

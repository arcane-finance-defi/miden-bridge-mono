import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { SolanaScanModel } from 'src/models/solana-scan.model';
import { EntityManager, Repository } from 'typeorm';

@Injectable()
export class SolscansService {
  constructor(
    @InjectRepository(SolanaScanModel)
    private readonly inner: Repository<SolanaScanModel>,
  ) {}

  async insertScan(
    model: Partial<SolanaScanModel>,
    manager: EntityManager = this.inner.manager,
  ): Promise<string> {
    const result = await manager
      .getRepository(SolanaScanModel)
      .createQueryBuilder()
      .insert()
      .values(model)
      .returning(['id'])
      .execute();

    return result.raw[0].id;
  }
}

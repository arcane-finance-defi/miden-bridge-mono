import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { ExitModel } from 'src/models/exit.model';
import { FulfillModel } from 'src/models/fulfill.model';
import { EntityManager, IsNull, Repository } from 'typeorm';

@Injectable()
export class ExitRepository {
  constructor(
    @InjectRepository(ExitModel)
    private readonly inner: Repository<ExitModel>,
  ) {}

  tx<T>(payload: (em: EntityManager) => Promise<T>): Promise<T> {
    return this.inner.manager.transaction(payload);
  }

  async insertExit(
    model: Partial<ExitModel>,
    manager: EntityManager = this.inner.manager,
  ): Promise<void> {
    await manager.getRepository(ExitModel).insert(model);
  }

  async getPendingExitsPage(
    pageSize: number,
    pageNum: number,
    manager: EntityManager = this.inner.manager,
  ): Promise<[Array<ExitModel>, number]> {
    return manager
      .getRepository(ExitModel)
      .createQueryBuilder('e')
      .leftJoin(FulfillModel, 'f', 'e.id = f.exit_id')
      .where('f.id IS NULL')
      .take(pageSize)
      .skip(pageNum * pageSize)
      .getManyAndCount();
  }

  async fulfill(
    ids: Array<string>,
    manager: EntityManager = this.inner.manager,
  ): Promise<void> {
    await manager.getRepository(FulfillModel).insert(
      ids.map((id) => ({
        exitId: id,
      })),
    );
  }
}

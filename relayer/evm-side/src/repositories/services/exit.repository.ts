import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { ExitModel } from 'src/models/exit.model';
import { EntityManager, Repository } from 'typeorm';

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

  async;
}

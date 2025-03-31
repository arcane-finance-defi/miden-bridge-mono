import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ExitModel } from 'src/models/exit.model';
import { ScanModel } from 'src/models/scan.model';
import { ExitRepository } from './services/exit.repository';
import { ScansRepository } from './services/scans.repository';

@Module({
  imports: [TypeOrmModule.forFeature([ExitModel, ScanModel])],
  providers: [ExitRepository, ScansRepository],
  exports: [ExitRepository, ScansRepository],
})
export class RepositoriesModule {}

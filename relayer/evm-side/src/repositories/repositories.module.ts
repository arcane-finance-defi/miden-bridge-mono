import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ExitModel } from 'src/models/exit.model';
import { ScanModel } from 'src/models/scan.model';
import { ExitRepository } from './services/exit.repository';
import { ScansRepository } from './services/scans.repository';
import { SolscansService } from './services/solscans.repository';
import { SolanaScanModel } from 'src/models/solana-scan.model';

@Module({
  imports: [TypeOrmModule.forFeature([ExitModel, ScanModel, SolanaScanModel])],
  providers: [ExitRepository, ScansRepository, SolscansService],
  exports: [ExitRepository, ScansRepository, SolscansService],
})
export class RepositoriesModule {}

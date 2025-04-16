import { Module } from '@nestjs/common';
import { MainConfigModule } from 'src/config';
import { RepositoriesModule } from 'src/repositories/repositories.module';
import { RelayerService } from './services/relayer.service';
import { MidenModule } from '../miden';

@Module({
  imports: [MidenModule, MainConfigModule, RepositoriesModule],
  providers: [RelayerService],
})
export class RelayerModule {}

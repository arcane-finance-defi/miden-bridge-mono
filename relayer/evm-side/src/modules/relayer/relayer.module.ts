import { Module } from '@nestjs/common';
import { MainConfigModule } from 'src/config';
import { RepositoriesModule } from 'src/repositories/repositories.module';
import { MidenModule } from '../miden';
import { MidenRelayerService } from './services/miden.service';
import { EVMRelayerService } from './services/evm.service';
import { RelayerService } from './services/relayer.service';

@Module({
  imports: [MidenModule, MainConfigModule, RepositoriesModule],
  providers: [RelayerService, MidenRelayerService, EVMRelayerService],
})
export class RelayerModule {}

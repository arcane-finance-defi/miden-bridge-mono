import { Module } from '@nestjs/common';
import { MainConfigModule } from 'src/config';
import { MidenApiService } from './services/miden.service';

@Module({
  imports: [MainConfigModule],
  providers: [MidenApiService],
  exports: [MidenApiService],
})
export class MidenModule {}

import { Module } from '@nestjs/common';
import { KindService } from './kind/kind.service';
import { MainConfigModule } from 'src/config';

@Module({
  imports: [MainConfigModule],
  providers: [KindService],
  exports: [KindService],
})
export class ChainsModule {}

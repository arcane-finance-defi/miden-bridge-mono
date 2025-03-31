import { Module } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';

import { MainConfigService } from './services';
import { RpcConfigService } from './services/rpc.service';
import { EVM_BRIDGE_ADDRESSES, EVM_RPCS, MIDEN_RPCS } from './config.const';
import { BridgeAddressService } from './services/bridge-address.service';

@Module({
  imports: [],
  providers: [
    MainConfigService,
    {
      provide: EVM_RPCS,
      useFactory(main: MainConfigService, config: ConfigService) {
        return RpcConfigService.connectEvmChains(main.getEvmChainIds(), config);
      },
      inject: [MainConfigService, ConfigService],
    },
    {
      provide: MIDEN_RPCS,
      useFactory(main: MainConfigService, config: ConfigService) {
        return RpcConfigService.connectMidenChains(
          main.getMidenChainIds(),
          config,
        );
      },
      inject: [MainConfigService, ConfigService],
    },
    {
      provide: EVM_BRIDGE_ADDRESSES,
      useFactory(main: MainConfigService, config: ConfigService) {
        return BridgeAddressService.getBridgeAddress(
          main.getEvmChainIds(),
          config,
        );
      },
      inject: [MainConfigService, ConfigService],
    },
  ],
  exports: [MainConfigService, EVM_RPCS, MIDEN_RPCS, EVM_BRIDGE_ADDRESSES],
})
export class MainConfigModule {}

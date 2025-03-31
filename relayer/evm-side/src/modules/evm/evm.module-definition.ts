import { ConfigurableModuleBuilder } from '@nestjs/common';
import { EvmModuleOptions } from './interfaces/evm-module-options.interface';

export const { ConfigurableModuleClass, MODULE_OPTIONS_TOKEN } =
  new ConfigurableModuleBuilder<EvmModuleOptions>().build();

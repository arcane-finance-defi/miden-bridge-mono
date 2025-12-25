import { ConfigurableModuleBuilder } from '@nestjs/common';
import { SolanaModuleOptions } from './interfaces/solana-module-options.interface';

export const { ConfigurableModuleClass, MODULE_OPTIONS_TOKEN } =
  new ConfigurableModuleBuilder<SolanaModuleOptions>().build();

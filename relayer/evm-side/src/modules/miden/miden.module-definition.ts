import { ConfigurableModuleBuilder } from '@nestjs/common';
import { MidenModuleOptions } from './interfaces/miden-module-options.interface';

export const { ConfigurableModuleClass, MODULE_OPTIONS_TOKEN } =
  new ConfigurableModuleBuilder<MidenModuleOptions>().build();

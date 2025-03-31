import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { TypeOrmModule } from '@nestjs/typeorm';
import { dataSourceOptions } from 'db/typeOrm.config';
import { EnvConfigSchema, MainConfigModule, MainConfigService } from './config';
import { WinstonModule } from 'nest-winston';
import { format, transports } from 'winston';
import { EvmModule } from './modules/evm/evm.module';
import { ScheduleModule } from '@nestjs/schedule';
import { RepositoriesModule } from './repositories/repositories.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      validationSchema: EnvConfigSchema,
      //настройка конфигурации из .env файла
      envFilePath: `${process.cwd()}/.env`,
      isGlobal: true,
    }),
    TypeOrmModule.forRootAsync({
      useFactory: () => dataSourceOptions,
    }),
    WinstonModule.forRoot({
      level: process.env.LOG_LEVEL || 'debug',
      format: format.combine(
        format.colorize({ all: true }),
        format.simple(),
        format.printf((info) => {
          return `[${info.level}] ${info.message} ${(info?.error as any)?.stack || ''}`;
        }),
      ),
      transports: [new transports.Console()],
    }),
    MainConfigModule,
    EvmModule.registerAsync({
      imports: [MainConfigModule],
      inject: [MainConfigService],
      useFactory(config: MainConfigService) {
        return {
          chainIds: config.getEvmChainIds(),
        };
      },
    }),
    ScheduleModule.forRoot(),
    RepositoriesModule,
  ],
})
export class AppModule {}

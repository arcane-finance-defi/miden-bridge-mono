import { ConfigService } from '@nestjs/config';
import { config } from 'dotenv';
import { join } from 'path';
import { DataSource, DataSourceOptions } from 'typeorm';

config();

const configService = new ConfigService();

const url = configService.get('POSTGRES_URL');
const ca = configService.get('POSTGRES_CERT');

if (!url) {
  throw new Error('Not found env POSTGRES_URL');
}

export const dataSourceOptions: DataSourceOptions = {
  type: 'postgres',
  url,
  logging: true,
  entities: [
    join(__dirname, '../src/models/*.model.ts'),
    join(__dirname, '../src/models/*.model.js'),
    join(__dirname, '!../src/models/*.model.js.map'),
  ],
  migrations: [
    join(__dirname, './migrations/*.migration.ts'),
    join(__dirname, './migrations/*.migration.js'),
    join(__dirname, '!./migrations/*.migration.js.map'),
  ],
  migrationsRun: true,
  ssl: ca ? { ca } : false,
};

const dataSource = new DataSource(dataSourceOptions);

export default dataSource;

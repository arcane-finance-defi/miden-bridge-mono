import { MigrationInterface, QueryRunner } from 'typeorm';

export class ExitsTable implements MigrationInterface {
  name = 'ExitsTable1742212969044';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TABLE "exits" ("id" SERIAL NOT NULL, "asset_address" text NOT NULL, "amount" numeric NOT NULL, "asset_metadata_name" text NOT NULL, "asset_metadata_symbol" text NOT NULL, "asset_metadata_decimals" integer NOT NULL, "asset_metadata_max_supply" numeric, "transaction_id" text NOT NULL, "from_Chain_id" integer NOT NULL, "from_Chain_kind" character varying NOT NULL, "to_Chain_id" integer NOT NULL, "to_Chain_kind" character varying NOT NULL, "assetOrigin_Chain_id" integer NOT NULL, "assetOrigin_Chain_kind" character varying NOT NULL, CONSTRAINT "PK_bc10e84eb866599a06689b2c4e5" PRIMARY KEY ("id"))`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`DROP TABLE "exits"`);
  }
}

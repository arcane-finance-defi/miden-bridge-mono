import { MigrationInterface, QueryRunner } from 'typeorm';

export class ScansTable1742305362098 implements MigrationInterface {
  name = 'ScansTable1742305362098';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TABLE "chain_scans" ("id" SERIAL NOT NULL, "start_block" integer NOT NULL, "end_block" integer NOT NULL, "created_at" TIMESTAMP NOT NULL DEFAULT now(), "chain_Chain_id" integer NOT NULL, "chain_Chain_kind" character varying NOT NULL, CONSTRAINT "PK_db786bd34ccedc2a1a906c0930f" PRIMARY KEY ("id"))`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`DROP TABLE "chain_scans"`);
  }
}

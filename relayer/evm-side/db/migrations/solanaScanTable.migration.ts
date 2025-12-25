import { MigrationInterface, QueryRunner } from 'typeorm';

export class SolanaScanTable implements MigrationInterface {
  name = 'SolanaScanTable1762624273921';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TABLE "solana_scans" ("id" SERIAL NOT NULL, "earliest_signature" text NOT NULL, "latest_signature" text NOT NULL, "scan_id" integer, CONSTRAINT "REL_0ea72b5cbb64879e5a45a34161" UNIQUE ("scan_id"), CONSTRAINT "PK_8beeec5e0bc894e3dc1cbf26bf7" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(
      `ALTER TABLE "solana_scans" ADD CONSTRAINT "FK_0ea72b5cbb64879e5a45a34161a" FOREIGN KEY ("scan_id") REFERENCES "chain_scans"("id") ON DELETE NO ACTION ON UPDATE NO ACTION`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "solana_scans" DROP CONSTRAINT "FK_0ea72b5cbb64879e5a45a34161a"`,
    );
    await queryRunner.query(`DROP TABLE "solana_scans"`);
  }
}

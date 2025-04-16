import { MigrationInterface, QueryRunner } from 'typeorm';

export class ExitFields1744649244467 implements MigrationInterface {
  name = 'ExitFields1744649244467';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`ALTER TABLE "exits" ADD "call_address" text`);
    await queryRunner.query(
      `ALTER TABLE "exits" ADD "block_number" integer NOT NULL`,
    );
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "transaction_id" DROP NOT NULL`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "transaction_id" SET NOT NULL`,
    );
    await queryRunner.query(`ALTER TABLE "exits" DROP COLUMN "block_number"`);
    await queryRunner.query(`ALTER TABLE "exits" DROP COLUMN "call_address"`);
  }
}

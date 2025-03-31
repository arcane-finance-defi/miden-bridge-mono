import { MigrationInterface, QueryRunner } from 'typeorm';

export class EixtAssetMetadataOptional1742308380162
  implements MigrationInterface
{
  name = 'EixtAssetMetadataOptional1742308380162';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "exits" DROP COLUMN "asset_metadata_max_supply"`,
    );
    await queryRunner.query(`ALTER TABLE "exits" ADD "sender" text`);
    await queryRunner.query(`ALTER TABLE "exits" ADD "receiver" text`);
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "asset_metadata_name" DROP NOT NULL`,
    );
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "asset_metadata_symbol" DROP NOT NULL`,
    );
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "asset_metadata_decimals" DROP NOT NULL`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "asset_metadata_decimals" SET NOT NULL`,
    );
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "asset_metadata_symbol" SET NOT NULL`,
    );
    await queryRunner.query(
      `ALTER TABLE "exits" ALTER COLUMN "asset_metadata_name" SET NOT NULL`,
    );
    await queryRunner.query(`ALTER TABLE "exits" DROP COLUMN "receiver"`);
    await queryRunner.query(`ALTER TABLE "exits" DROP COLUMN "sender"`);
    await queryRunner.query(
      `ALTER TABLE "exits" ADD "asset_metadata_max_supply" numeric`,
    );
  }
}

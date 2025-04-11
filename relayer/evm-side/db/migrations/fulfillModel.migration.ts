import { MigrationInterface, QueryRunner } from 'typeorm';

export class FulfillModel implements MigrationInterface {
  name = 'FulfillModel1744373575619';

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TABLE "fulfills" ("id" SERIAL NOT NULL, "exit_id" integer NOT NULL, CONSTRAINT "REL_55264ba3eda919cb8b5054f173" UNIQUE ("exit_id"), CONSTRAINT "PK_8ad8d5c8edee49a6da282c519cc" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(`ALTER TABLE "exits" ADD "calldata" bytea`);
    await queryRunner.query(
      `ALTER TABLE "fulfills" ADD CONSTRAINT "FK_55264ba3eda919cb8b5054f1737" FOREIGN KEY ("exit_id") REFERENCES "exits"("id") ON DELETE NO ACTION ON UPDATE NO ACTION`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "fulfills" DROP CONSTRAINT "FK_55264ba3eda919cb8b5054f1737"`,
    );
    await queryRunner.query(`ALTER TABLE "exits" DROP COLUMN "calldata"`);
    await queryRunner.query(`DROP TABLE "fulfills"`);
  }
}

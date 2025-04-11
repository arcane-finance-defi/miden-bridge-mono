import {
  Column,
  Entity,
  JoinColumn,
  OneToOne,
  PrimaryGeneratedColumn,
} from 'typeorm';
import { ExitModel } from './exit.model';

@Entity({ name: 'fulfills' })
export class FulfillModel {
  @PrimaryGeneratedColumn()
  id!: string;

  @Column({ name: 'exit_id' })
  exitId: string;

  @OneToOne(() => ExitModel, { nullable: false })
  @JoinColumn({ name: 'exit_id', referencedColumnName: 'id' })
  exit: ExitModel;
}

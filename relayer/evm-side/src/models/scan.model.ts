import {
  Column,
  CreateDateColumn,
  Entity,
  PrimaryGeneratedColumn,
} from 'typeorm';
import { ChainRef } from './exit.model';

@Entity('chain_scans')
export class ScanModel {
  @PrimaryGeneratedColumn()
  id: string;

  @Column(() => ChainRef, { prefix: 'chain_' })
  chain: ChainRef;

  @Column({ name: 'start_block', type: 'int' })
  startBlock: number;

  @Column({ name: 'end_block', type: 'int' })
  endBlock: number;

  @CreateDateColumn({ name: 'created_at' })
  createdAt: Date;
}

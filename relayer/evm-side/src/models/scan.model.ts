import {
  Column,
  CreateDateColumn,
  Entity,
  OneToOne,
  PrimaryGeneratedColumn,
} from 'typeorm';
import { ChainRef } from './exit.model';
import { SolanaScanModel } from './solana-scan.model';

@Entity('chain_scans')
export class ScanModel {
  @PrimaryGeneratedColumn()
  id: string;

  @Column(() => ChainRef, { prefix: 'chain_' })
  chain: ChainRef;

  @Column({ name: 'start_block', type: 'int' })
  startBlock: number;

  @OneToOne(() => SolanaScanModel, (details) => details.scan, {
    nullable: true,
  })
  solanaScan?: SolanaScanModel | null;

  @Column({ name: 'end_block', type: 'int' })
  endBlock: number;

  @CreateDateColumn({ name: 'created_at' })
  createdAt: Date;
}

import {
  Column,
  Entity,
  JoinColumn,
  OneToOne,
  PrimaryGeneratedColumn,
} from 'typeorm';
import { ScanModel } from './scan.model';

@Entity('solana_scans')
export class SolanaScanModel {
  @PrimaryGeneratedColumn()
  id: string;

  @OneToOne(() => ScanModel)
  @JoinColumn({ name: 'scan_id', referencedColumnName: 'id' })
  scan: ScanModel;

  @Column({ name: 'earliest_signature', type: 'text' })
  earliestSignature: string;

  @Column({ name: 'latest_signature', type: 'text', nullable: true })
  latestSignature: string;
}

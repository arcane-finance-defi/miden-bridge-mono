import BigNumber from 'bignumber.js';
import { Column, Entity, PrimaryGeneratedColumn } from 'typeorm';

type ChainKind = 'evm' | 'miden';

export class ChainRef {
  @Column({
    name: 'chain_id',
    type: 'int',
    transformer: {
      from(value: string) {
        return BigInt(value);
      },
      to(value: bigint) {
        return value.toString();
      },
    },
  })
  chainId!: bigint;

  @Column({
    name: 'chain_kind',
    enum: ['evm', 'miden'],
    enumName: 'networkKind',
  })
  chainKind: ChainKind;
}

@Entity({ name: 'exits' })
export class ExitModel {
  @PrimaryGeneratedColumn({ name: 'id' })
  id!: string;

  @Column(() => ChainRef, { prefix: 'from_' })
  from: ChainRef;

  @Column(() => ChainRef, { prefix: 'to_' })
  to: ChainRef;

  @Column({ name: 'sender', type: 'text', nullable: true })
  sender: string;

  @Column({ name: 'receiver', type: 'text', nullable: true })
  receiver: string;

  @Column(() => ChainRef, { prefix: 'asset_origin_' })
  assetOrigin: ChainRef;

  @Column({ name: 'asset_address', type: 'text' })
  assetAddress!: string;

  @Column({
    name: 'amount',
    type: 'decimal',
    transformer: {
      from(value: string) {
        return new BigNumber(value);
      },
      to(value: BigNumber) {
        return value.toFixed();
      },
    },
  })
  assetAmount: BigNumber;

  @Column({ name: 'asset_metadata_name', type: 'text', nullable: true })
  assetName?: string;

  @Column({ name: 'asset_metadata_symbol', type: 'text', nullable: true })
  assetSymbol!: string;

  @Column({ name: 'asset_metadata_decimals', type: 'int', nullable: true })
  assetDecimals!: number;

  @Column({
    name: 'calldata',
    nullable: true,
    type: 'bytea',
    transformer: {
      from(value: Buffer) {
        return '0x' + value.toString('hex');
      },
      to(value: string) {
        return Buffer.from(value.replace('0x', ''), 'hex');
      },
    },
  })
  calldata?: string;

  @Column({ name: 'call_address', type: 'text', nullable: true })
  callAddress?: string;

  @Column({ name: 'transaction_id', type: 'text', nullable: true })
  txId?: string;

  @Column({ name: 'block_number', type: 'int', nullable: false })
  blockNumber!: number;
}

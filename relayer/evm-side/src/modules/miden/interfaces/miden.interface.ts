import BigNumber from 'bignumber.js';

export interface AssetMetadata {
  symbol: string;
  decimals: number;
}

export interface AssetOrigin {
  network: number;
  address: string;
}

export interface SendRequest {
  asset: AssetMetadata & AssetOrigin;
  recipient: string;
  amount: BigNumber;
}

export interface SendResponse {
  noteId: string;
  faucetId: string;
  transactionId: string;
}

export interface BridgeEvent {
  leafType: 0 | 1;
  originNetwork: bigint;
  originAddress: string;
  destinationNetwork: bigint;
  destinationAddress: string;
  amount: bigint;
  metadata?: any;
  depositCount: bigint;
  tx: string;
  blockNumber: number;
}

export interface BridgeMessageMetadata {
  dependsOnIndex: bigint;
  callAddress: string;
  fallbackAddress: string;
  assetOriginalNetwork: bigint;
  assetOriginalAddr: string;
  callData: string;
}

export interface BridgeAssetMetadata {
  name: string;
  symbol: string;
  deimals: number;
}

export type AssetBridgeEvent = Omit<BridgeEvent, 'leafType' | 'metadata'> & {
  leafType: 0;
  metadata: BridgeAssetMetadata;
};
export type MessageBridgeEvent = Omit<BridgeEvent, 'leafType' | 'metadata'> & {
  leafType: 1;
  metadata: BridgeMessageMetadata;
};

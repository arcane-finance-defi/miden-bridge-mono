import { Inject } from '@nestjs/common';
import { AbiCoder, Provider, ZeroAddress } from 'ethers';
import { EVM_BRIDGE_ADDRESSES, EVM_RPCS } from 'src/config';
import {
  AssetBridgeEvent,
  BridgeAssetMetadata,
  BridgeEvent,
  BridgeMessageMetadata,
  MessageBridgeEvent,
} from '../interfaces/bridge-event.interface';
import { BRIDGE_EVENT_ABI } from '../evm.abi';

const BRIDGE_EVENT_SIGNATURE =
  '0x501781209a1f8899323b96b4ef08b168df93e0a90c673d1e4cce39366cb62f9b';

function decodeMessageMetadata(metadata: string): BridgeMessageMetadata {
  const [dependsOnIndex, assetOriginalNetwork, assetOriginalAddr, callData] =
    AbiCoder.defaultAbiCoder().decode(
      ['uint256', 'uint32', 'address', 'bytes'],
      Buffer.from(metadata.replace('0x', ''), 'hex'),
    );

  return {
    dependsOnIndex: BigInt(dependsOnIndex),
    callAddress: ZeroAddress,
    fallbackAddress: ZeroAddress,
    assetOriginalNetwork: BigInt(assetOriginalNetwork),
    assetOriginalAddr,
    callData,
  };
}

function decodeAssetMetadata(metadata: string): BridgeAssetMetadata {
  const [name, symbol, decimals] = AbiCoder.defaultAbiCoder().decode(
    ['string', 'string', 'uint8'],
    Buffer.from(metadata.replace('0x', ''), 'hex'),
  );

  return {
    name,
    symbol,
    deimals: Number.parseInt(decimals),
  };
}

export class RpcService {
  constructor(
    @Inject(EVM_RPCS) private readonly providers: Map<bigint, Provider>,
    @Inject(EVM_BRIDGE_ADDRESSES) private readonly bridges: Map<bigint, string>,
  ) {}

  async getChainHeight(chainId: bigint): Promise<number> {
    return this.providers.get(chainId).getBlockNumber();
  }

  async getBridgeEvents(
    chainId: bigint,
    blockStart: number,
    blockFinish: number,
  ): Promise<Array<[AssetBridgeEvent, MessageBridgeEvent]>> {
    const provider = this.providers.get(chainId);
    const bridgeAddress = this.bridges.get(chainId);

    const logs = await provider.getLogs({
      address: bridgeAddress,
      topics: [BRIDGE_EVENT_SIGNATURE],
      fromBlock: blockStart,
      toBlock: blockFinish,
    });

    const events = logs
      .map((log) => {
        const decoded = BRIDGE_EVENT_ABI.parseLog(log);
        if (decoded) {
          return {
            decoded,
            tx: log.transactionHash,
            blockNumber: log.blockNumber,
          };
        }
        return null;
      })
      .filter(
        (result) => result !== null && result.decoded.name === 'BridgeEvent',
      )
      .map<BridgeEvent>(({ decoded, tx, blockNumber }) => {
        const leafType = Number.parseInt(decoded.args[0]) as 0 | 1;
        let metadata = null;
        try {
          metadata =
            decoded.args[6] != null && decoded.args[6] !== '0x'
              ? leafType === 1
                ? decodeMessageMetadata(decoded.args[6])
                : decodeAssetMetadata(decoded.args[6])
              : null;
        } catch (err) {
          // ignore
        }
        return {
          leafType,
          originNetwork: BigInt(decoded.args[1]),
          originAddress: decoded.args[2],
          destinationNetwork: BigInt(decoded.args[3]),
          destinationAddress: decoded.args[4],
          amount: BigInt(decoded.args[5]),
          metadata,
          depositCount: BigInt(decoded.args[7]),
          tx,
          blockNumber,
        };
      });

    const assets: Array<AssetBridgeEvent> = events.filter(
      (e) => e.leafType === 0,
    ) as any;
    const messages: Array<MessageBridgeEvent> = events.filter(
      (e) => e.leafType === 1,
    ) as any;

    return assets
      .map<
        [AssetBridgeEvent, MessageBridgeEvent]
      >((asset) => [asset, messages.find((message) => message.metadata != null && message.metadata.dependsOnIndex === asset.depositCount + 1n)])
      .filter((pair) => pair[1] != null);
  }
}

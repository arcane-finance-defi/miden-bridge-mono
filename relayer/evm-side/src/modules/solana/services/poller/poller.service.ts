import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import {
  Address,
  address,
  Rpc,
  Signature,
  signature,
  SolanaRpcApi,
} from '@solana/kit';
import {
  getSolanaAssetOriginAddressEnvVarKey,
  getSolanaAssetDecimalsEnvVarKey,
  getSolanaAssetNameEnvVarKey,
  getSolanaAssetOriginNetworkEnvVarKey,
  getSolanaAssetSymbolEnvVarKey,
  getSolanaBridgeAddressEnvVarKey,
  getSolanaBridgeFirstSignatureEnvVarKey,
  getSolanaStartBlockScanEnvVarKey,
  MainConfigService,
} from 'src/config';
import { ExitRepository, ScansRepository } from 'src/repositories';
import { SolscansService } from 'src/repositories/services/solscans.repository';
import { parseLogs } from '@debridge-finance/solana-transaction-parser';
import {
  getSwapOutEventDecoder,
  SwapOutEvent,
} from '../../../relayer/generated/solana/types';
import { ExitModel } from 'src/models/exit.model';
import BigNumber from 'bignumber.js';
import { KindService } from 'src/modules/chains/kind/kind.service';
import { ZeroAddress } from 'ethers';

const SWAP_OUT_EVENT_DISCRIMINATOR = Buffer.from([
  21, 117, 255, 136, 215, 209, 131, 32,
]);

@Injectable()
export class PollerService {
  private readonly assetOriginNetwork: bigint;
  private readonly assetOriginAddress: string;
  private readonly assetDecimals: number;
  private readonly assetSymbol: string;
  private readonly assetName: string;
  private readonly logger: Logger;
  private readonly bridgeAddress: string;
  private readonly firstSignature: string;

  constructor(
    private readonly chainId: bigint,
    private readonly scans: ScansRepository,
    private readonly solscans: SolscansService,
    private readonly exits: ExitRepository,
    private readonly config: MainConfigService,
    private readonly rpc: Rpc<SolanaRpcApi>,
    private readonly kindService: KindService,
  ) {
    this.logger = new Logger(`SolanaPollerService (${chainId})`);
    this.bridgeAddress = this.config.getSolanaChainConf(
      getSolanaBridgeAddressEnvVarKey(this.chainId),
    );
    this.assetOriginNetwork = BigInt(
      this.config.getSolanaChainConf(
        getSolanaAssetOriginNetworkEnvVarKey(this.chainId, this.bridgeAddress),
      ),
    );
    this.assetOriginAddress = this.config.getSolanaChainConf(
      getSolanaAssetOriginAddressEnvVarKey(this.chainId, this.bridgeAddress),
    );
    this.assetDecimals = Number(
      this.config.getSolanaChainConf(
        getSolanaAssetDecimalsEnvVarKey(this.chainId, this.bridgeAddress),
      ),
    );
    this.assetSymbol = this.config.getSolanaChainConf(
      getSolanaAssetSymbolEnvVarKey(this.chainId, this.bridgeAddress),
    );
    this.assetName = this.config.getSolanaChainConf(
      getSolanaAssetNameEnvVarKey(this.chainId, this.bridgeAddress),
    );
    this.firstSignature = this.config.getSolanaChainConf(
      getSolanaBridgeFirstSignatureEnvVarKey(this.chainId, this.bridgeAddress),
    );
    if (
      this.bridgeAddress == null ||
      this.assetOriginNetwork == null ||
      this.assetOriginAddress == null ||
      this.assetDecimals == null ||
      this.assetSymbol == null ||
      this.assetName == null ||
      this.firstSignature == null
    ) {
      throw new Error(
        `Asset origin network, address or decimals not found for chain ${this.chainId}`,
      );
    }
  }

  private async processSignature(
    rpc: Rpc<SolanaRpcApi>,
    signature: Signature,
    programId: Address,
  ) {
    const transaction = await rpc
      .getTransaction(signature, {
        encoding: 'jsonParsed',
        commitment: 'confirmed',
        maxSupportedTransactionVersion: 0,
      })
      .send();
    const parsedLogs = parseLogs(
      (transaction.meta.logMessages as string[]) || [],
    );
    const decoder = getSwapOutEventDecoder();
    const relevantLogs = parsedLogs.filter(
      (log) => log.programId === programId,
    );

    const decodedEvents: Array<SwapOutEvent & { txId: string }> = [];
    for (const log of relevantLogs) {
      if (log.dataLogs) {
        try {
          const bytes = Buffer.from(log.dataLogs[0], 'base64');
          if (
            bytes.subarray(0, 8).toString('hex') !==
            SWAP_OUT_EVENT_DISCRIMINATOR.toString('hex')
          ) {
            continue;
          }
          const decoded = decoder.decode(bytes.subarray(8));
          if (decoded) {
            decodedEvents.push({
              ...decoded,
              txId: signature.toString(),
            });
          }
        } catch (error) {
          // Silent error handling
        }
      }
    }

    const exits: Array<{ exit: Partial<ExitModel>; signature: string }> =
      decodedEvents.map((event) => ({
        exit: {
          txId: event.txId,
          assetAddress: this.assetOriginAddress,
          assetOrigin: {
            chainId: this.assetOriginNetwork,
            chainKind: this.kindService.getChainKind(this.assetOriginNetwork),
          },
          assetAmount: new BigNumber(event.amount.toString()),
          from: {
            chainId: this.chainId,
            chainKind: 'solana',
          },
          to: {
            chainId: BigInt(event.destChain),
            chainKind: this.kindService.getChainKind(event.destChain),
          },
          sender: '0x' + Buffer.from(event.from).toString('hex'),
          receiver:
            this.kindService.getChainKind(event.destChain) === 'miden'
              ? ZeroAddress
              : '0x' + Buffer.from(event.to).toString('hex'),
          calldata: Buffer.from(event.to).toString('hex'),
          assetName: this.assetName,
          assetSymbol: this.assetSymbol,
          assetDecimals: this.assetDecimals,
          blockNumber: Number.parseInt(transaction.slot.toString()),
        },
        signature: signature.toString(),
      }));

    return exits;
  }

  private async processGap(
    gap: {
      latestSignature: string | null;
      earliestSignature: string | null;
    },
    rpc: Rpc<SolanaRpcApi>,
    bridgeContract: Address,
  ): Promise<{
    signatures: { earliest: Signature | null; latest: Signature | null };
    exits: Array<Partial<ExitModel>>;
  }> {
    const { latestSignature, earliestSignature } = gap;
    const signatures = await rpc
      .getSignaturesForAddress(bridgeContract, {
        commitment: 'confirmed',
        limit: 1000, // TODO handle limit overload
        before:
          latestSignature && latestSignature !== earliestSignature
            ? signature(latestSignature)
            : undefined,
        until: earliestSignature ? signature(earliestSignature) : undefined,
      })
      .send();
    if (signatures.length > 0) {
      const sortedSignatures = signatures
        .slice()
        .sort((a, b) => Number(a.blockTime - b.blockTime));

      const result = await this.processSignature(
        rpc,
        sortedSignatures[0].signature,
        bridgeContract,
      );

      return {
        signatures: {
          earliest: signatures[0].signature,
          latest: signatures[signatures.length - 1].signature,
        },
        exits: result.map(({ exit }) => exit),
      };
    }
    return {
      signatures: {
        earliest: earliestSignature ? signature(earliestSignature) : null,
        latest: latestSignature ? signature(latestSignature) : null,
      },
      exits: [],
    };
  }

  @Cron(CronExpression.EVERY_10_SECONDS, { waitForCompletion: true })
  async poll() {
    this.logger.log(`Polling solana chain ${this.chainId}`);
    const startScanBlockHeight = Number.parseInt(
      this.config.getSolanaChainConf(
        getSolanaStartBlockScanEnvVarKey(this.chainId),
      ) || '0',
    );
    const bridgeContract = address(
      this.config.getSolanaChainConf(
        getSolanaBridgeAddressEnvVarKey(this.chainId),
      ),
    );

    const lastChainBlockHeight = await this.rpc
      .getBlockHeight({
        commitment: 'confirmed',
      })
      .send();

    const gaps = await this.scans.getScanGapsFor(
      this.chainId,
      startScanBlockHeight,
      lastChainBlockHeight,
    );
    this.logger.log(
      `Found ${gaps.length} gaps for solana chain ${this.chainId}`,
    );
    await this.exits.tx(async (em) => {
      for (const gap of gaps) {
        const processedGap = await this.processGap(
          {
            latestSignature: gap.latestSignature,
            earliestSignature: gap.earliestSignature || this.firstSignature,
          },
          this.rpc,
          bridgeContract,
        );
        for (const exit of processedGap.exits) {
          await this.exits.insertExit(exit, em);
        }
        const scanId = await this.scans.insertScan(
          {
            chain: {
              chainId: this.chainId,
              chainKind: 'solana',
            },
            startBlock: gap.startBlock,
            endBlock: gap.endBlock,
          },
          em,
        );
        const earliestSignature =
          processedGap.signatures.earliest?.toString() || this.firstSignature;
        await this.solscans.insertScan(
          {
            scan: {
              id: scanId,
            } as any,
            latestSignature:
              processedGap.signatures.latest?.toString() || earliestSignature,
            earliestSignature,
          },
          em,
        );
        this.logger.log(`Scan to block ${lastChainBlockHeight} saved`);
      }
    });
    this.logger.log(`Polling solana chain ${this.chainId} completed`);
  }
}

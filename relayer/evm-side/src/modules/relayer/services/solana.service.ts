import { Inject, Injectable, Logger } from '@nestjs/common';
import {
  getSolanaBridgeAddressEnvVarKey,
  getSolanaBridgeUserPrivateKeyEnvVarKey,
  MainConfigService,
  SOLANA_RPCS,
} from 'src/config';
import { ExitModel } from 'src/models/exit.model';
import {
  fetchBridgeConfig,
  getSubmitMessageInstruction,
  getSubmitSignatureInstruction,
  SwapPayload,
} from '../generated/solana';
import {
  address,
  getAddressDecoder,
  Rpc,
  SolanaRpcApi,
  pipe,
  createTransactionMessage,
  appendTransactionMessageInstructions,
  Instruction,
  getAddressEncoder,
  appendTransactionMessageInstruction,
  setTransactionMessageFeePayerSigner,
  signTransactionMessageWithSigners,
  getBase64EncodedWireTransaction,
  createKeyPairSignerFromBytes,
  setTransactionMessageLifetimeUsingBlockhash,
} from '@solana/kit';
import {
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
} from '@solana-program/token';
import {
  estimateComputeUnitLimitFactory,
  getSetComputeUnitLimitInstruction,
} from '@solana-program/compute-budget';
import {
  calculateMessageHash,
  createEd25519InstructionData,
  CrossChainPayload,
  ED25519_PROGRAM_ID,
  getConfigPDAAddress,
  getMintAuthorityPDA,
  getSwapMessagePDAAddress,
} from '../utils/solana';
import * as nacl from 'tweetnacl';

@Injectable()
export class SVMRelayerService {
  private readonly logger = new Logger(SVMRelayerService.name);

  constructor(
    @Inject(SOLANA_RPCS)
    private readonly rpcs: Map<bigint, Rpc<SolanaRpcApi>>,
    private readonly config: MainConfigService,
  ) {}

  async relay(exit: ExitModel) {
    const rpc = this.rpcs.get(exit.to.chainId);
    const bridgeContract = address(
      this.config.getSolanaChainConf(
        getSolanaBridgeAddressEnvVarKey(exit.to.chainId),
      ),
    );
    if (rpc == null || bridgeContract == null) {
      throw new Error(`Unknown Solana chain with id ${exit.to.chainId}`);
    }
    const privateKey = this.config.getSolanaChainConf(
      getSolanaBridgeUserPrivateKeyEnvVarKey(exit.to.chainId, bridgeContract),
    );
    if (privateKey == null) {
      throw new Error(
        `Private key not found for bridge contract ${bridgeContract}`,
      );
    }
    const user = await createKeyPairSignerFromBytes(
      Buffer.from(privateKey, 'hex'),
    );

    const payload: SwapPayload = {
      amount: BigInt(exit.assetAmount.toFixed()),
      recipient: getAddressDecoder().decode(
        Buffer.from(exit.receiver.replace('0x', ''), 'hex'),
      ),
      sourceChain: Number.parseInt(exit.from.chainId.toString()) % 2 ** 16,
      destChain: Number.parseInt(exit.to.chainId.toString()),
      nonce: BigInt(exit.id),
    };

    const crossChainPayload: CrossChainPayload = {
      sender:
        exit.from.chainKind === 'miden'
          ? Buffer.alloc(32)
          : Buffer.from(exit.sender.replace('0x', ''), 'hex'),
      to: Buffer.from(exit.receiver.replace('0x', ''), 'hex'),
      amount: payload.amount,
      dest_chain: payload.destChain,
      nonce: payload.nonce,
    };

    const config = await fetchBridgeConfig(
      rpc,
      address(await getConfigPDAAddress(bridgeContract)),
    );

    const messageHash = calculateMessageHash(crossChainPayload);
    const messagePDA = await getSwapMessagePDAAddress(
      messageHash,
      address(bridgeContract),
    );

    const [recipientTokenAccount] = await findAssociatedTokenPda({
      owner: payload.recipient,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
      mint: config.data.bridgeMint,
    });

    const signature = nacl.sign.detached(
      messageHash,
      Buffer.from(privateKey, 'hex'),
    );

    const mintAuthority = await getMintAuthorityPDA(bridgeContract);

    const instructions: Array<Instruction> = [
      {
        programAddress: ED25519_PROGRAM_ID,
        accounts: [],
        data: createEd25519InstructionData(
          getAddressEncoder().encode(user.address),
          messageHash,
          signature,
        ),
      },
      getSubmitMessageInstruction({
        swapMessage: messagePDA,
        payload,
        messageHash,
        originalSender: crossChainPayload.sender,
        user,
      }),
      getSubmitSignatureInstruction({
        config: config.address,
        bridgeMint: config.data.bridgeMint,
        swapMessage: messagePDA,
        recipientTokenAccount,
        recipient: payload.recipient,
        payer: user,
        signerPubkey: user.address,
        mintAuthority,
      }),
    ];

    const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    const transactionMessage = pipe(
      createTransactionMessage({ version: 0 }),
      (tx) => setTransactionMessageFeePayerSigner(user, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) => appendTransactionMessageInstructions(instructions, tx),
    );

    const estimateComputeUnitLimit = estimateComputeUnitLimitFactory({
      rpc,
    });

    // Estimate compute units.
    const computeUnitsEstimate =
      await estimateComputeUnitLimit(transactionMessage);

    const transactionMessageWithComputeUnitLimit = pipe(
      transactionMessage,
      (tx) =>
        appendTransactionMessageInstruction(
          getSetComputeUnitLimitInstruction({ units: computeUnitsEstimate }),
          tx,
        ),
    );

    const transaction = await signTransactionMessageWithSigners(
      transactionMessageWithComputeUnitLimit,
    );
    const encodedTransaction = getBase64EncodedWireTransaction(transaction);

    const pendingTx = await rpc
      .sendTransaction(encodedTransaction, {
        preflightCommitment: 'confirmed',
        encoding: 'base64',
      })
      .send();

    this.logger.debug(`Transaction sent with hash ${pendingTx}`);
  }
}

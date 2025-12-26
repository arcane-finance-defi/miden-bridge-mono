import {
  address,
  Address,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
  getStructEncoder,
  getU16Encoder,
  getU32Encoder,
  getU64Encoder,
  ReadonlyUint8Array,
} from '@solana/kit';
import { TransactionInstruction, Ed25519Program } from '@solana/web3.js';
import { createHash } from 'node:crypto';

export interface CrossChainPayload {
  sender: Address;
  to: Address;
  amount: bigint;
  dest_chain: number;
  nonce: bigint;
}

function getCrossChainPayloadEncoder() {
  return getStructEncoder([
    ['sender', getAddressEncoder()],
    ['to', getAddressEncoder()],
    ['amount', getU64Encoder()],
    ['dest_chain', getU32Encoder()],
    ['nonce', getU64Encoder()],
  ]);
}

// Function to calculate the correct message hash
export const calculateMessageHash = (
  crossChainPayload: CrossChainPayload,
): Uint8Array => {
  const encoder = getCrossChainPayloadEncoder();
  const buffer = Buffer.from(encoder.encode(crossChainPayload));

  // Calculate SHA-256 hash (Solana uses hashv which is effectively SHA-256)
  const hash = createHash('sha256').update(buffer).digest();
  return new Uint8Array(hash);
};

export async function getSwapMessagePDAAddress(
  messageHash: Uint8Array,
  programId: Address,
): Promise<Address> {
  const seeds = [Buffer.from('swap_message'), messageHash];

  const [address] = await getProgramDerivedAddress({
    seeds,
    programAddress: programId,
  });
  return address;
}

export async function getConfigPDAAddress(
  programId: Address,
): Promise<Address> {
  const seeds = [Buffer.from('bridge_config')];
  const [address] = await getProgramDerivedAddress({
    seeds,
    programAddress: programId,
  });
  return address;
}

export async function getMintAuthorityPDA(programId: Address) {
  const [derivedPDA] = await getProgramDerivedAddress({
    seeds: [Buffer.from('mint_authority')],
    programAddress: programId,
  });
  return derivedPDA;
}

export function createEd25519Instruction(
  publicKey: Uint8Array,
  message: Uint8Array,
  signature: Uint8Array,
): TransactionInstruction {
  return Ed25519Program.createInstructionWithPublicKey({
    publicKey,
    message,
    signature,
  });
}

export const ED25519_PROGRAM_ID = address(
  'Ed25519SigVerify111111111111111111111111111',
);

interface Ed25519SignatureOffsets {
  signature_offset: number; // offset to ed25519 signature of 64 bytes
  signature_instruction_index: number; // instruction index to find signature
  public_key_offset: number; // offset to public key of 32 bytes
  public_key_instruction_index: number; // instruction index to find public key
  message_data_offset: number; // offset to start of message data
  message_data_size: number; // size of message data
  message_instruction_index: number; // index of instruction data to get message data
  signature: ReadonlyUint8Array; // 64 bytes
  public_key: ReadonlyUint8Array; // 32 bytes
  message: ReadonlyUint8Array; // your message
}

function getEd25519SignaturesOffsetEncodes() {
  return getStructEncoder([
    ['signature_offset', getU16Encoder()],
    ['signature_instruction_index', getU16Encoder()],
    ['public_key_offset', getU16Encoder()],
    ['public_key_instruction_index', getU16Encoder()],
    ['message_data_offset', getU16Encoder()],
    ['message_data_size', getU16Encoder()],
    ['message_instruction_index', getU16Encoder()],
    ['public_key', getBytesEncoder()],
    ['signature', getBytesEncoder()],
    ['message', getBytesEncoder()],
  ]);
}

export function createEd25519InstructionData(
  publicKey: ReadonlyUint8Array, // 32 bytes
  message: Uint8Array, // your message
  signature: Uint8Array, // 64 bytes
): ReadonlyUint8Array {
  const signatureInstructionIndex = 65535;
  const dataEncoder = getEd25519SignaturesOffsetEncodes();
  const dataStart = 1 + 1 + 14; // 1 byte num_signatures + 1 byte padding + 14 bytes offsets

  const publicKeyOffset = dataStart;
  const signatureOffset = publicKeyOffset + publicKey.length;
  const messageOffset = signatureOffset + signature.length;

  const instructionData: Ed25519SignatureOffsets = {
    signature_offset: signatureOffset,
    signature_instruction_index: signatureInstructionIndex,
    public_key_offset: publicKeyOffset,
    public_key_instruction_index: signatureInstructionIndex,
    message_data_offset: messageOffset,
    message_data_size: message.length,
    message_instruction_index: signatureInstructionIndex,
    signature: signature,
    public_key: publicKey,
    message: message,
  };

  return Uint8Array.from([1, 0, ...dataEncoder.encode(instructionData)]);
}

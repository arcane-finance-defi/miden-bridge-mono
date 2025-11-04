import {
  address,
  Address,
  getBytesEncoder,
  getProgramDerivedAddress,
  getStructEncoder,
  getU16Encoder,
  ReadonlyUint8Array,
} from '@solana/kit';
import { TransactionInstruction, Ed25519Program } from '@solana/web3.js';
import { createHash } from 'node:crypto';

export interface CrossChainPayload {
  sender: Uint8Array;
  to: Uint8Array;
  amount: bigint;
  dest_chain: number;
  nonce: bigint;
}

function addressToBuffer(address: Buffer | Uint8Array | number[]): Buffer {
  if (Buffer.isBuffer(address)) {
    return address;
  } else if (address instanceof Uint8Array) {
    return Buffer.from(address);
  } else if (Array.isArray(address)) {
    return Buffer.from(address);
  }
  throw new Error('Invalid address format');
}

// Function to calculate the correct message hash
export const calculateMessageHash = (
  crossChainPayload: CrossChainPayload,
): Uint8Array => {
  // Serialize the CrossChainPayload similar to Borsh serialization
  const buffer = Buffer.alloc(32 + 32 + 8 + 2 + 8); // [u8;32] + [u8;32] + u64 + u16 + u64
  let offset = 0;

  // sender: [u8; 32]
  const senderBuf = addressToBuffer(crossChainPayload.sender);
  senderBuf.copy(buffer, offset);
  offset += 32;

  // to: [u8; 32]
  const toBuf = addressToBuffer(crossChainPayload.to);
  toBuf.copy(buffer, offset);
  offset += 32;

  // amount: u64 (8 bytes, little endian)
  const amountBuf = Buffer.alloc(8);
  amountBuf.writeBigUInt64LE(BigInt(crossChainPayload.amount.toString()), 0);
  amountBuf.copy(buffer, offset);
  offset += 8;

  // dest_chain: u16 (2 bytes, little endian)
  buffer.writeUInt16LE(crossChainPayload.dest_chain, offset);
  offset += 2;

  // nonce: u64 (8 bytes, little endian)
  const nonceBuf = Buffer.alloc(8);
  nonceBuf.writeBigUInt64LE(BigInt(crossChainPayload.nonce.toString()), 0);
  nonceBuf.copy(buffer, offset);

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
    ['signature', getBytesEncoder()],
    ['public_key', getBytesEncoder()],
    ['message', getBytesEncoder()],
  ]);
}

export function createEd25519InstructionData(
  publicKey: ReadonlyUint8Array, // 32 bytes
  message: Uint8Array, // your message
  signature: Uint8Array, // 64 bytes
  instructionIndex: number = 0,
): ReadonlyUint8Array {
  const dataEncoder = getEd25519SignaturesOffsetEncodes();
  const dataStart = 1 + 1 + 14; // 1 byte num_signatures + 1 byte padding + 14 bytes offsets

  const signatureOffset = dataStart;
  const publicKeyOffset = signatureOffset + signature.length;
  const messageOffset = publicKeyOffset + publicKey.length;

  const instructionData: Ed25519SignatureOffsets = {
    signature_offset: signatureOffset,
    signature_instruction_index: instructionIndex,
    public_key_offset: publicKeyOffset,
    public_key_instruction_index: instructionIndex,
    message_data_offset: messageOffset,
    message_data_size: message.length,
    message_instruction_index: instructionIndex,
    signature: signature,
    public_key: publicKey,
    message: message,
  };

  return Uint8Array.from([1, 0, ...dataEncoder.encode(instructionData)]);
}

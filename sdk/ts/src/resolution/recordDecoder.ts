import { PublicKey } from '@solana/web3.js';
import { SRS_RECORD_DISCRIMINATOR } from './constants';
import { SrsRecordDecodeError } from './errors';

const PUBKEY_BYTES = 32;

export interface DecodedSrsRecord {
  discriminator: number;
  class: PublicKey;
  ownerType: number;
  owner: PublicKey;
  isFrozen: boolean;
  expiry: bigint;
  seed: Uint8Array;
  data: Uint8Array;
}

function readPublicKey(raw: Uint8Array, offset: number): PublicKey {
  const end = offset + PUBKEY_BYTES;
  if (end > raw.length) {
    throw new SrsRecordDecodeError('Unexpected EOF while reading public key');
  }

  return new PublicKey(raw.subarray(offset, end));
}

function readBigInt64LE(raw: Uint8Array, offset: number): bigint {
  if (offset + 8 > raw.length) {
    throw new SrsRecordDecodeError('Unexpected EOF while reading i64');
  }

  const view = new DataView(raw.buffer, raw.byteOffset, raw.byteLength);
  return view.getBigInt64(offset, true);
}

export function decodeSrsRecord(rawAccountData: Uint8Array): DecodedSrsRecord {
  const minimumHeaderLength = 1 + 32 + 1 + 32 + 1 + 8 + 1;
  if (rawAccountData.length < minimumHeaderLength) {
    throw new SrsRecordDecodeError('SRS record is too short');
  }

  let offset = 0;
  const discriminator = rawAccountData[offset];
  if (discriminator === undefined) {
    throw new SrsRecordDecodeError('Missing discriminator');
  }
  offset += 1;

  if (discriminator !== SRS_RECORD_DISCRIMINATOR) {
    throw new SrsRecordDecodeError(
      `Unexpected record discriminator: ${discriminator}`
    );
  }

  const classPk = readPublicKey(rawAccountData, offset);
  offset += PUBKEY_BYTES;

  const ownerType = rawAccountData[offset];
  if (ownerType === undefined) {
    throw new SrsRecordDecodeError('Missing owner type');
  }
  offset += 1;

  const ownerPk = readPublicKey(rawAccountData, offset);
  offset += PUBKEY_BYTES;

  const frozen = rawAccountData[offset];
  if (frozen === undefined) {
    throw new SrsRecordDecodeError('Missing frozen flag');
  }
  offset += 1;

  if (frozen !== 0 && frozen !== 1) {
    throw new SrsRecordDecodeError('Invalid boolean frozen value');
  }

  const expiry = readBigInt64LE(rawAccountData, offset);
  offset += 8;

  const seedLength = rawAccountData[offset];
  if (seedLength === undefined) {
    throw new SrsRecordDecodeError('Missing seed length');
  }
  offset += 1;

  const seedEnd = offset + seedLength;
  if (seedEnd > rawAccountData.length) {
    throw new SrsRecordDecodeError('Unexpected EOF while reading seed');
  }

  const seed = rawAccountData.subarray(offset, seedEnd);
  const data = rawAccountData.subarray(seedEnd);

  return {
    discriminator,
    class: classPk,
    ownerType,
    owner: ownerPk,
    isFrozen: frozen === 1,
    expiry,
    seed,
    data,
  };
}

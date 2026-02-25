import { PublicKey } from '@solana/web3.js';

export function deterministicPublicKey(seedByte: number): PublicKey {
  const bytes = new Uint8Array(32);
  bytes.fill(seedByte);
  return new PublicKey(bytes);
}

export interface BuildSrsRecordOptions {
  classAddress: PublicKey;
  ownerAddress: PublicKey;
  seed: Uint8Array;
  data: Uint8Array;
  ownerType?: number;
  isFrozen?: boolean;
  expiry?: bigint;
}

export function buildSrsRecordAccountData(
  options: BuildSrsRecordOptions
): Uint8Array {
  if (options.seed.length > 0xff) {
    throw new Error('seed must fit in u8');
  }

  const fixedLength = 1 + 32 + 1 + 32 + 1 + 8 + 1;
  const out = new Uint8Array(fixedLength + options.seed.length + options.data.length);
  const view = new DataView(out.buffer);

  let offset = 0;
  out[offset++] = 2;

  out.set(options.classAddress.toBytes(), offset);
  offset += 32;

  out[offset++] = options.ownerType ?? 0;

  out.set(options.ownerAddress.toBytes(), offset);
  offset += 32;

  out[offset++] = options.isFrozen ? 1 : 0;
  view.setBigInt64(offset, options.expiry ?? 0n, true);
  offset += 8;

  out[offset++] = options.seed.length;
  out.set(options.seed, offset);
  offset += options.seed.length;

  out.set(options.data, offset);
  return out;
}

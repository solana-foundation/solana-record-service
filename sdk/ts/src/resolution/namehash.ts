import { keccak_256 } from '@noble/hashes/sha3';
import { ResolutionInputError } from './errors';

const textEncoder = new TextEncoder();
const tr46: {
  toASCII: (name: string, options?: Record<string, unknown>) => string | null;
} = require('tr46');

function concat32(left: Uint8Array, right: Uint8Array): Uint8Array {
  const out = new Uint8Array(64);
  out.set(left, 0);
  out.set(right, 32);
  return out;
}

export function normalizeName(name: string): string {
  const trimmed = name.trim();
  if (trimmed.length === 0) {
    throw new ResolutionInputError('Name cannot be empty');
  }

  const normalized = tr46.toASCII(trimmed, {
    checkBidi: true,
    checkHyphens: true,
    useSTD3ASCIIRules: true,
    verifyDNSLength: true,
    transitionalProcessing: false,
  });

  if (!normalized) {
    throw new ResolutionInputError(`Could not normalize name: ${name}`);
  }

  return normalized.toLowerCase();
}

export function namehash(name: string): Uint8Array {
  const normalized = normalizeName(name);
  if (normalized === '.') {
    return new Uint8Array(32);
  }

  const labels = normalized.split('.').filter((label) => label.length > 0);
  let node = new Uint8Array(32);

  for (let i = labels.length - 1; i >= 0; i -= 1) {
    const labelBytes = textEncoder.encode(labels[i]);
    const labelHash = Uint8Array.from(keccak_256(labelBytes));
    node = Uint8Array.from(keccak_256(concat32(node, labelHash)));
  }

  return node;
}

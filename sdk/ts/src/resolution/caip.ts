import { ResolutionInputError } from './errors';

const CAIP2_PATTERN = /^[a-z0-9]{3,8}:[a-zA-Z0-9-]{1,32}$/;

export interface ParsedWalletValue {
  chainId: string;
  walletAddress: string;
}

export function normalizeChainCaip2(chainId: string): string {
  const trimmed = chainId.trim();
  if (!CAIP2_PATTERN.test(trimmed)) {
    throw new ResolutionInputError(`Invalid CAIP-2 chain id: ${chainId}`);
  }

  return trimmed.toLowerCase();
}

function parseCaipWalletValue(value: string): ParsedWalletValue | null {
  const trimmed = value.trim();
  const parts = trimmed.split(':');

  if (parts.length < 3) {
    return null;
  }

  // did:pkh:<namespace>:<reference>:<account>
  if (parts[0] === 'did' && parts[1] === 'pkh' && parts.length >= 5) {
    const chain = `${parts[2]}:${parts[3]}`;
    if (!CAIP2_PATTERN.test(chain)) {
      return null;
    }

    const walletAddress = parts.slice(4).join(':').trim();
    if (walletAddress.length === 0) {
      return null;
    }

    return { chainId: chain.toLowerCase(), walletAddress };
  }

  const chainId = `${parts[0]}:${parts[1]}`;
  if (!CAIP2_PATTERN.test(chainId)) {
    return null;
  }

  const walletAddress = parts.slice(2).join(':').trim();
  if (walletAddress.length === 0) {
    return null;
  }

  return { chainId: chainId.toLowerCase(), walletAddress };
}

export function parseWalletTuple(
  key: string,
  value: string,
  defaultChainCaip2: string
): ParsedWalletValue | null {
  const normalizedDefault = normalizeChainCaip2(defaultChainCaip2);
  const normalizedKey = key.trim().toUpperCase();

  if (normalizedKey === 'WALLET') {
    const parsed = parseCaipWalletValue(value);
    if (parsed) {
      return parsed;
    }

    const walletAddress = value.trim();
    if (walletAddress.length === 0) {
      return null;
    }

    return { chainId: normalizedDefault, walletAddress };
  }

  if (!normalizedKey.startsWith('WALLET:')) {
    return null;
  }

  const chainInKeyRaw = key.slice('WALLET:'.length).trim();
  if (chainInKeyRaw.length === 0) {
    return null;
  }

  const chainId = normalizeChainCaip2(chainInKeyRaw);
  const parsed = parseCaipWalletValue(value);
  if (parsed) {
    return {
      chainId,
      walletAddress: parsed.walletAddress,
    };
  }

  const walletAddress = value.trim();
  if (walletAddress.length === 0) {
    return null;
  }

  return { chainId, walletAddress };
}

import { PublicKey } from '@solana/web3.js';
import { normalizeChainCaip2 } from './caip';
import { DEFAULT_SOLANA_CAIP2, SRS_DEFAULT_PROGRAM_ID } from './constants';
import { ResolutionInputError } from './errors';
import { normalizeName } from './namehash';
import { findRecordPda, reverseRecordSeed } from './pda';
import type { RawRecordAccountProvider } from './provider';
import { decodeSrsRecord } from './recordDecoder';
import { parseResolutionTuples } from './tupleCodec';

export interface ForwardNameResolver {
  resolve(name: string, chainCaip2?: string): Promise<string | null>;
}

export interface SrsReverseResolverConfig {
  provider: RawRecordAccountProvider;
  reverseClassAddress: PublicKey;
  programId?: PublicKey;
  defaultChainCaip2?: string;
  verifyReverseWithForward?: boolean;
  forwardVerifier?: ForwardNameResolver;
}

export class SrsReverseResolver {
  private readonly provider: RawRecordAccountProvider;
  private readonly reverseClassAddress: PublicKey;
  private readonly programId: PublicKey;
  private readonly defaultChainCaip2: string;
  private readonly verifyReverseWithForward: boolean;
  private readonly forwardVerifier?: ForwardNameResolver;

  constructor(config: SrsReverseResolverConfig) {
    this.provider = config.provider;
    this.reverseClassAddress = config.reverseClassAddress;
    this.programId = config.programId ?? SRS_DEFAULT_PROGRAM_ID;
    this.defaultChainCaip2 = normalizeChainCaip2(
      config.defaultChainCaip2 ?? DEFAULT_SOLANA_CAIP2
    );
    this.verifyReverseWithForward = config.verifyReverseWithForward ?? true;
    this.forwardVerifier = config.forwardVerifier;

    if (this.verifyReverseWithForward && !this.forwardVerifier) {
      throw new ResolutionInputError(
        'forwardVerifier is required when verifyReverseWithForward is enabled'
      );
    }
  }

  async reverseResolve(wallet: string): Promise<string | null> {
    const reverseSeed = reverseRecordSeed(wallet);
    const [recordPda] = findRecordPda(
      this.reverseClassAddress,
      reverseSeed,
      this.programId
    );

    const tuples = await this.fetchRecordTuples(recordPda);
    if (!tuples) {
      return null;
    }

    let selectedName: string | null = null;
    for (const [key, value] of tuples) {
      if (key.trim().toUpperCase() !== 'NAME') {
        continue;
      }

      const candidateName = value.trim();
      if (candidateName.length > 0) {
        selectedName = normalizeName(candidateName);
      }
    }

    if (!selectedName) {
      return null;
    }

    if (!this.verifyReverseWithForward) {
      return selectedName;
    }

    const resolvedWallet = await this.forwardVerifier?.resolve(
      selectedName,
      this.defaultChainCaip2
    );
    if (!resolvedWallet) {
      return null;
    }

    return resolvedWallet === wallet ? selectedName : null;
  }

  async batchReverseResolve(
    wallets: readonly string[]
  ): Promise<Array<string | null>> {
    return Promise.all(wallets.map((wallet) => this.reverseResolve(wallet)));
  }

  private async fetchRecordTuples(recordPda: PublicKey) {
    const raw = await this.provider.fetchRawRecordAccount(recordPda);
    if (!raw) {
      return null;
    }

    const decoded = decodeSrsRecord(raw);
    return parseResolutionTuples(decoded.data);
  }
}

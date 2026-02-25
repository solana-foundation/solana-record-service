import { PublicKey } from '@solana/web3.js';
import { ResolutionInputError } from './errors';
import {
  DomaForwardResolver,
  type DomaForwardResolverConfig,
} from './forwardResolver';
import {
  SrsReverseResolver,
  type SrsReverseResolverConfig,
} from './reverseResolver';

export interface DomaSrsResolverConfig {
  provider: DomaForwardResolverConfig['provider'];
  domaClassAddress?: PublicKey;
  // Backward-compatible alias for older call sites.
  forwardClassAddress?: PublicKey;
  reverseClassAddress: PublicKey;
  programId?: PublicKey;
  defaultChainCaip2?: string;
  verifyReverseWithForward?: boolean;
}

export class DomaSrsResolver {
  private readonly forwardResolver: DomaForwardResolver;
  private readonly reverseResolver: SrsReverseResolver;

  constructor(config: DomaSrsResolverConfig) {
    const domaClassAddress = config.domaClassAddress ?? config.forwardClassAddress;
    if (!domaClassAddress) {
      throw new ResolutionInputError('domaClassAddress is required');
    }

    this.forwardResolver = new DomaForwardResolver({
      provider: config.provider,
      domaClassAddress,
      programId: config.programId,
      defaultChainCaip2: config.defaultChainCaip2,
    });

    const reverseConfig: SrsReverseResolverConfig = {
      provider: config.provider,
      reverseClassAddress: config.reverseClassAddress,
      programId: config.programId,
      defaultChainCaip2: config.defaultChainCaip2,
      verifyReverseWithForward: config.verifyReverseWithForward,
      forwardVerifier: this.forwardResolver,
    };

    this.reverseResolver = new SrsReverseResolver(reverseConfig);
  }

  async resolve(name: string, chainCaip2?: string): Promise<string | null> {
    return this.forwardResolver.resolve(name, chainCaip2);
  }

  async reverseResolve(wallet: string): Promise<string | null> {
    return this.reverseResolver.reverseResolve(wallet);
  }

  async batchReverseResolve(
    wallets: readonly string[]
  ): Promise<Array<string | null>> {
    return this.reverseResolver.batchReverseResolve(wallets);
  }
}

import { expect } from 'chai';
import { PublicKey } from '@solana/web3.js';
import {
  decodeSrsRecord,
  DomaForwardResolver,
  DomaSrsResolver,
  findRecordPda,
  namehash,
  parseResolutionTuples,
  reverseRecordSeed,
  serializeResolutionTuples,
  SrsReverseResolver,
  type RawRecordAccountProvider,
} from '../src/resolution';
import { buildSrsRecordAccountData, deterministicPublicKey } from './helpers';

class InMemoryRecordProvider implements RawRecordAccountProvider {
  private readonly records = new Map<string, Uint8Array>();

  put(recordPda: PublicKey, data: Uint8Array): void {
    this.records.set(recordPda.toBase58(), data);
  }

  async fetchRawRecordAccount(recordPda: PublicKey): Promise<Uint8Array | null> {
    return this.records.get(recordPda.toBase58()) ?? null;
  }
}

describe('resolution codec', () => {
  it('serializes and parses resolution tuples', () => {
    const tuples = [
      ['WALLET:solana:mainnet', deterministicPublicKey(88).toBase58()],
      ['NAME', 'alice.sol'],
    ] as const;

    const encoded = serializeResolutionTuples(tuples);
    const decoded = parseResolutionTuples(encoded);

    expect(decoded).to.deep.equal(tuples);
  });

  it('emits UTF-8 payload compatible with current SRS string record data', () => {
    const tuples = [
      ['WALLET', deterministicPublicKey(89).toBase58()],
      ['NAME', 'alice.sol'],
    ] as const;

    const encoded = serializeResolutionTuples(tuples);
    const decodedUtf8 = new TextDecoder('utf-8', { fatal: true }).decode(encoded);

    expect(decodedUtf8.length).to.be.greaterThan(0);
    expect(parseResolutionTuples(encoded)).to.deep.equal(tuples);
  });

  it('decodes SRS account data and extracts tuple payload', () => {
    const classAddress = deterministicPublicKey(20);
    const ownerAddress = deterministicPublicKey(21);
    const seed = namehash('alice.sol');
    const tuples = serializeResolutionTuples([['NAME', 'alice.sol']]);

    const rawRecord = buildSrsRecordAccountData({
      classAddress,
      ownerAddress,
      seed,
      data: tuples,
      expiry: 123n,
    });

    const decoded = decodeSrsRecord(rawRecord);
    expect(decoded.class.toBase58()).to.equal(classAddress.toBase58());
    expect(decoded.owner.toBase58()).to.equal(ownerAddress.toBase58());
    expect(decoded.expiry).to.equal(123n);
    expect(parseResolutionTuples(decoded.data)).to.deep.equal([['NAME', 'alice.sol']]);
  });
});

describe('DomaForwardResolver', () => {
  it('resolves forward records with chain-aware last-write-wins', async () => {
    const domaClassAddress = deterministicPublicKey(30);
    const ownerAddress = deterministicPublicKey(32);
    const latestWallet = deterministicPublicKey(33).toBase58();

    const provider = new InMemoryRecordProvider();
    const resolver = new DomaForwardResolver({
      provider,
      domaClassAddress,
      defaultChainCaip2: 'solana:mainnet',
    });

    const seed = namehash('alice.sol');
    const [recordPda] = findRecordPda(domaClassAddress, seed);

    provider.put(
      recordPda,
      buildSrsRecordAccountData({
        classAddress: domaClassAddress,
        ownerAddress,
        seed,
        data: serializeResolutionTuples([
          ['WALLET:solana:mainnet', deterministicPublicKey(34).toBase58()],
          ['WALLET:eip155:1', '0x123'],
          ['WALLET', latestWallet],
        ]),
      })
    );

    const resolved = await resolver.resolve('alice.sol');
    expect(resolved).to.equal(latestWallet);
  });

  it('resolves token-owned forward records and supports DID-PKH wallet values', async () => {
    const domaClassAddress = deterministicPublicKey(35);
    const tokenOwnerAddress = deterministicPublicKey(37);
    const wallet = deterministicPublicKey(38).toBase58();

    const provider = new InMemoryRecordProvider();
    const resolver = new DomaForwardResolver({
      provider,
      domaClassAddress,
      defaultChainCaip2: 'solana:mainnet',
    });

    const seed = namehash('tokenized.sol');
    const [recordPda] = findRecordPda(domaClassAddress, seed);

    provider.put(
      recordPda,
      buildSrsRecordAccountData({
        classAddress: domaClassAddress,
        ownerAddress: tokenOwnerAddress,
        ownerType: 1,
        seed,
        data: serializeResolutionTuples([
          ['WALLET', `did:pkh:solana:mainnet:${wallet}`],
        ]),
      })
    );

    const resolved = await resolver.resolve('tokenized.sol');
    expect(resolved).to.equal(wallet);
  });

  it('supports CAIP-10 wallet values', async () => {
    const domaClassAddress = deterministicPublicKey(45);
    const ownerAddress = deterministicPublicKey(46);
    const wallet = deterministicPublicKey(47).toBase58();

    const provider = new InMemoryRecordProvider();
    const resolver = new DomaForwardResolver({
      provider,
      domaClassAddress,
      defaultChainCaip2: 'solana:mainnet',
    });

    const seed = namehash('caip10.sol');
    const [recordPda] = findRecordPda(domaClassAddress, seed);

    provider.put(
      recordPda,
      buildSrsRecordAccountData({
        classAddress: domaClassAddress,
        ownerAddress,
        seed,
        data: serializeResolutionTuples([
          ['WALLET', `solana:mainnet:${wallet}`],
        ]),
      })
    );

    const resolved = await resolver.resolve('caip10.sol');
    expect(resolved).to.equal(wallet);
  });
});

describe('SrsReverseResolver', () => {
  it('requires forward verifier when verification is enabled by default', () => {
    const provider = new InMemoryRecordProvider();

    expect(
      () =>
        new SrsReverseResolver({
          provider,
          reverseClassAddress: deterministicPublicKey(90),
        })
    ).to.throw('forwardVerifier is required when verifyReverseWithForward is enabled');
  });

  it('reverse resolves name only when forward mapping matches', async () => {
    const domaClassAddress = deterministicPublicKey(40);
    const reverseClassAddress = deterministicPublicKey(41);
    const ownerAddress = deterministicPublicKey(42);
    const wallet = deterministicPublicKey(43).toBase58();
    const name = 'alice.sol';

    const provider = new InMemoryRecordProvider();
    const forwardResolver = new DomaForwardResolver({
      provider,
      domaClassAddress,
    });
    const reverseResolver = new SrsReverseResolver({
      provider,
      reverseClassAddress,
      forwardVerifier: forwardResolver,
      verifyReverseWithForward: true,
    });

    const forwardSeed = namehash(name);
    const [forwardPda] = findRecordPda(domaClassAddress, forwardSeed);
    provider.put(
      forwardPda,
      buildSrsRecordAccountData({
        classAddress: domaClassAddress,
        ownerAddress,
        seed: forwardSeed,
        data: serializeResolutionTuples([['WALLET', wallet]]),
      })
    );

    const reverseSeed = reverseRecordSeed(wallet);
    const [reversePda] = findRecordPda(reverseClassAddress, reverseSeed);
    provider.put(
      reversePda,
      buildSrsRecordAccountData({
        classAddress: reverseClassAddress,
        ownerAddress,
        seed: reverseSeed,
        data: serializeResolutionTuples([['NAME', 'ALICE.sol']]),
      })
    );

    const reverse = await reverseResolver.reverseResolve(wallet);
    expect(reverse).to.equal(name);
  });

  it('returns null for reverse mapping mismatch', async () => {
    const domaClassAddress = deterministicPublicKey(50);
    const reverseClassAddress = deterministicPublicKey(51);
    const ownerAddress = deterministicPublicKey(52);
    const wallet = deterministicPublicKey(53).toBase58();

    const provider = new InMemoryRecordProvider();
    const forwardResolver = new DomaForwardResolver({
      provider,
      domaClassAddress,
    });
    const reverseResolver = new SrsReverseResolver({
      provider,
      reverseClassAddress,
      forwardVerifier: forwardResolver,
      verifyReverseWithForward: true,
    });

    const forwardSeed = namehash('alice.sol');
    const [forwardPda] = findRecordPda(domaClassAddress, forwardSeed);
    provider.put(
      forwardPda,
      buildSrsRecordAccountData({
        classAddress: domaClassAddress,
        ownerAddress,
        seed: forwardSeed,
        data: serializeResolutionTuples([
          ['WALLET', deterministicPublicKey(54).toBase58()],
        ]),
      })
    );

    const reverseSeed = reverseRecordSeed(wallet);
    const [reversePda] = findRecordPda(reverseClassAddress, reverseSeed);
    provider.put(
      reversePda,
      buildSrsRecordAccountData({
        classAddress: reverseClassAddress,
        ownerAddress,
        seed: reverseSeed,
        data: serializeResolutionTuples([['NAME', 'alice.sol']]),
      })
    );

    const reverse = await reverseResolver.reverseResolve(wallet);
    expect(reverse).to.equal(null);
  });

  it('batch reverse resolves wallets in order', async () => {
    const reverseClassAddress = deterministicPublicKey(61);
    const ownerAddress = deterministicPublicKey(62);

    const walletA = deterministicPublicKey(63).toBase58();
    const walletB = deterministicPublicKey(64).toBase58();

    const provider = new InMemoryRecordProvider();
    const reverseResolver = new SrsReverseResolver({
      provider,
      reverseClassAddress,
      verifyReverseWithForward: false,
    });

    const reverseSeedA = reverseRecordSeed(walletA);
    const reverseSeedB = reverseRecordSeed(walletB);
    const [reversePdaA] = findRecordPda(reverseClassAddress, reverseSeedA);
    const [reversePdaB] = findRecordPda(reverseClassAddress, reverseSeedB);

    provider.put(
      reversePdaA,
      buildSrsRecordAccountData({
        classAddress: reverseClassAddress,
        ownerAddress,
        seed: reverseSeedA,
        data: serializeResolutionTuples([['NAME', 'alice.sol']]),
      })
    );

    provider.put(
      reversePdaB,
      buildSrsRecordAccountData({
        classAddress: reverseClassAddress,
        ownerAddress,
        seed: reverseSeedB,
        data: serializeResolutionTuples([['NAME', 'bob.sol']]),
      })
    );

    const results = await reverseResolver.batchReverseResolve([walletA, walletB]);
    expect(results).to.deep.equal(['alice.sol', 'bob.sol']);
  });

  it('reverse resolves from standalone reverse class without forward dependency when verification disabled', async () => {
    const reverseClassAddress = deterministicPublicKey(71);
    const ownerAddress = deterministicPublicKey(72);
    const wallet = deterministicPublicKey(73).toBase58();

    const provider = new InMemoryRecordProvider();
    const reverseResolver = new SrsReverseResolver({
      provider,
      reverseClassAddress,
      verifyReverseWithForward: false,
    });

    const seed = reverseRecordSeed(wallet);
    const [reversePda] = findRecordPda(reverseClassAddress, seed);

    provider.put(
      reversePda,
      buildSrsRecordAccountData({
        classAddress: reverseClassAddress,
        ownerAddress,
        seed,
        data: serializeResolutionTuples([['NAME', 'standalone.sol']]),
      })
    );

    const resolved = await reverseResolver.reverseResolve(wallet);
    expect(resolved).to.equal('standalone.sol');
  });
});

describe('DomaSrsResolver compatibility facade', () => {
  it('supports legacy forwardClassAddress alias', async () => {
    const forwardClassAddress = deterministicPublicKey(101);
    const reverseClassAddress = deterministicPublicKey(102);
    const ownerAddress = deterministicPublicKey(103);

    const provider = new InMemoryRecordProvider();
    const resolver = new DomaSrsResolver({
      provider,
      forwardClassAddress,
      reverseClassAddress,
    });

    const seed = namehash('legacy.sol');
    const [forwardPda] = findRecordPda(forwardClassAddress, seed);

    provider.put(
      forwardPda,
      buildSrsRecordAccountData({
        classAddress: forwardClassAddress,
        ownerAddress,
        seed,
        data: serializeResolutionTuples([
          ['WALLET', deterministicPublicKey(104).toBase58()],
        ]),
      })
    );

    const resolved = await resolver.resolve('legacy.sol');
    expect(resolved).to.equal(deterministicPublicKey(104).toBase58());
  });
});

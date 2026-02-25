import { PublicKey } from '@solana/web3.js';
import { SRS_DEFAULT_PROGRAM_ID, SRS_RECORD_PDA_SEED } from './constants';
import { ResolutionInputError } from './errors';

export function findRecordPda(
  classAddress: PublicKey,
  tokenId: Uint8Array,
  programId: PublicKey = SRS_DEFAULT_PROGRAM_ID
): [PublicKey, number] {
  if (tokenId.length === 0 || tokenId.length > 32) {
    throw new ResolutionInputError(
      `tokenId must be in range [1, 32], got ${tokenId.length}`
    );
  }

  return PublicKey.findProgramAddressSync(
    [SRS_RECORD_PDA_SEED, classAddress.toBuffer(), Buffer.from(tokenId)],
    programId
  );
}

export function reverseRecordSeed(wallet: string): Uint8Array {
  const walletPk = new PublicKey(wallet);
  return walletPk.toBytes();
}

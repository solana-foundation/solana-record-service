import { PublicKey } from '@solana/web3.js';

export const SRS_DEFAULT_PROGRAM_ID = new PublicKey(
  'srsUi2TVUUCyGcZdopxJauk8ZBzgAaHHZCVUhm5ifPa'
);

export const SRS_RECORD_PDA_SEED = Buffer.from('record', 'utf8');
export const SRS_RECORD_DISCRIMINATOR = 2;
export const SRS_RECORD_TUPLE_VERSION = 1;
export const DEFAULT_SOLANA_CAIP2 = 'solana:mainnet';

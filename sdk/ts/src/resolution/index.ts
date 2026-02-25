export {
  DomaForwardResolver,
  type DomaForwardResolverConfig,
} from './forwardResolver';
export {
  SrsReverseResolver,
  type ForwardNameResolver,
  type SrsReverseResolverConfig,
} from './reverseResolver';
export { DomaSrsResolver, type DomaSrsResolverConfig } from './resolver';
export {
  RpcRawRecordAccountProvider,
  type RawRecordAccountProvider,
} from './provider';

export {
  serializeResolutionTuples,
  parseResolutionTuples,
  type ResolutionTuple,
} from './tupleCodec';

export { decodeSrsRecord, type DecodedSrsRecord } from './recordDecoder';
export { namehash, normalizeName } from './namehash';
export { findRecordPda, reverseRecordSeed } from './pda';
export {
  parseWalletTuple,
  normalizeChainCaip2,
  type ParsedWalletValue,
} from './caip';
export {
  ResolutionCodecError,
  ResolutionInputError,
  SrsRecordDecodeError,
} from './errors';
export {
  DEFAULT_SOLANA_CAIP2,
  SRS_DEFAULT_PROGRAM_ID,
  SRS_RECORD_PDA_SEED,
} from './constants';

/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  ClusterFilter,
  Context,
  Program,
  PublicKey,
} from '@metaplex-foundation/umi';
import {
  getSolanaRecordServiceErrorFromCode,
  getSolanaRecordServiceErrorFromName,
} from '../errors';

export const SOLANA_RECORD_SERVICE_PROGRAM_ID =
  'srsUi2TVUUCyGcZdopxJauk8ZBzgAaHHZCVUhm5ifPa' as PublicKey<'srsUi2TVUUCyGcZdopxJauk8ZBzgAaHHZCVUhm5ifPa'>;

export function createSolanaRecordServiceProgram(): Program {
  return {
    name: 'solanaRecordService',
    publicKey: SOLANA_RECORD_SERVICE_PROGRAM_ID,
    getErrorFromCode(code: number, cause?: Error) {
      return getSolanaRecordServiceErrorFromCode(code, this, cause);
    },
    getErrorFromName(name: string, cause?: Error) {
      return getSolanaRecordServiceErrorFromName(name, this, cause);
    },
    isOnCluster() {
      return true;
    },
  };
}

export function getSolanaRecordServiceProgram<T extends Program = Program>(
  context: Pick<Context, 'programs'>,
  clusterFilter?: ClusterFilter
): T {
  return context.programs.get<T>('solanaRecordService', clusterFilter);
}

export function getSolanaRecordServiceProgramId(
  context: Pick<Context, 'programs'>,
  clusterFilter?: ClusterFilter
): PublicKey {
  return context.programs.getPublicKey(
    'solanaRecordService',
    SOLANA_RECORD_SERVICE_PROGRAM_ID,
    clusterFilter
  );
}

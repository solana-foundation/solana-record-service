import { SRS_RECORD_TUPLE_VERSION } from './constants';
import { ResolutionCodecError } from './errors';

export type ResolutionTuple = readonly [key: string, value: string];

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder('utf-8', { fatal: true });

function ensureU16(value: number, label: string): void {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff) {
    throw new ResolutionCodecError(`${label} must fit into u16`);
  }
}

function writeU16(view: DataView, offset: number, value: number): number {
  view.setUint16(offset, value, true);
  return offset + 2;
}

function readU16(view: DataView, offset: number): number {
  if (offset + 2 > view.byteLength) {
    throw new ResolutionCodecError('Unexpected EOF while reading u16');
  }
  return view.getUint16(offset, true);
}

export function serializeResolutionTuples(
  tuples: readonly ResolutionTuple[]
): Uint8Array {
  ensureU16(tuples.length, 'tuple count');

  const encoded = tuples.map(([key, value]) => {
    const keyBytes = textEncoder.encode(key);
    const valueBytes = textEncoder.encode(value);
    ensureU16(keyBytes.length, 'key length');
    ensureU16(valueBytes.length, 'value length');
    return { keyBytes, valueBytes };
  });

  let total = 1 + 2;
  for (const tuple of encoded) {
    total += 2 + tuple.keyBytes.length + 2 + tuple.valueBytes.length;
  }

  const buffer = new ArrayBuffer(total);
  const view = new DataView(buffer);
  const out = new Uint8Array(buffer);

  let offset = 0;
  out[offset++] = SRS_RECORD_TUPLE_VERSION;
  offset = writeU16(view, offset, encoded.length);

  for (const tuple of encoded) {
    offset = writeU16(view, offset, tuple.keyBytes.length);
    out.set(tuple.keyBytes, offset);
    offset += tuple.keyBytes.length;

    offset = writeU16(view, offset, tuple.valueBytes.length);
    out.set(tuple.valueBytes, offset);
    offset += tuple.valueBytes.length;
  }

  return out;
}

export function parseResolutionTuples(data: Uint8Array): ResolutionTuple[] {
  if (data.length < 3) {
    throw new ResolutionCodecError('Tuple payload too short');
  }

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 0;
  const version = data[offset];
  if (version === undefined) {
    throw new ResolutionCodecError('Missing tuple payload version');
  }
  offset += 1;

  if (version !== SRS_RECORD_TUPLE_VERSION) {
    throw new ResolutionCodecError(`Unsupported tuple payload version: ${version}`);
  }

  const tupleCount = readU16(view, offset);
  offset += 2;

  const tuples: ResolutionTuple[] = [];
  for (let i = 0; i < tupleCount; i += 1) {
    const keyLen = readU16(view, offset);
    offset += 2;

    if (offset + keyLen > data.length) {
      throw new ResolutionCodecError('Unexpected EOF while reading key');
    }
    const key = textDecoder.decode(data.subarray(offset, offset + keyLen));
    offset += keyLen;

    const valueLen = readU16(view, offset);
    offset += 2;

    if (offset + valueLen > data.length) {
      throw new ResolutionCodecError('Unexpected EOF while reading value');
    }
    const value = textDecoder.decode(data.subarray(offset, offset + valueLen));
    offset += valueLen;

    tuples.push([key, value]);
  }

  if (offset !== data.length) {
    throw new ResolutionCodecError('Trailing bytes after tuple payload');
  }

  return tuples;
}

export class ResolutionCodecError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ResolutionCodecError';
  }
}

export class ResolutionInputError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ResolutionInputError';
  }
}

export class SrsRecordDecodeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'SrsRecordDecodeError';
  }
}

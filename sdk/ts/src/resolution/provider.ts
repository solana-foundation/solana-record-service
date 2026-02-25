import { Connection, PublicKey } from '@solana/web3.js';

export interface RawRecordAccountProvider {
  fetchRawRecordAccount(recordPda: PublicKey): Promise<Uint8Array | null>;
}

export class RpcRawRecordAccountProvider implements RawRecordAccountProvider {
  private readonly connection: Connection;
  private readonly commitment:
    | 'processed'
    | 'confirmed'
    | 'finalized'
    | undefined;

  constructor(
    connection: Connection,
    commitment?: 'processed' | 'confirmed' | 'finalized'
  ) {
    this.connection = connection;
    this.commitment = commitment;
  }

  async fetchRawRecordAccount(recordPda: PublicKey): Promise<Uint8Array | null> {
    const account = await this.connection.getAccountInfo(recordPda, this.commitment);
    return account?.data ?? null;
  }
}

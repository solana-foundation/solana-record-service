import * as program from "../sdk/ts/src/index";
import { createSolanaClient, createTransaction, generateKeyPairSigner, getExplorerLink, getProgramDerivedAddress, getSignatureFromTransaction, KeyPairSigner, signTransactionMessageWithSigners } from "gill";

describe('test', () => {
    // Generate the keypair first
    let authority: KeyPairSigner;
    let classAddress: string;
    let rpc: any;
    let sendAndConfirmTransaction: any;
    
    before(async () => {  // Use before hook for async setup
        authority = await generateKeyPairSigner();
                
        // Get the address from the public key for PDA derivation
        const name = "twitter";
        classAddress = (await getProgramDerivedAddress({
            programAddress: program.SOLANA_RECORD_SERVICE_PROGRAM_ADDRESS,
            seeds: [
                "class",
                authority.address.substring(0,32),
                name
            ]
        }))[0];

        const client = createSolanaClient({
            urlOrMoniker: "http://localhost:8899",
        });
        
        rpc = client.rpc;
        sendAndConfirmTransaction = client.sendAndConfirmTransaction;
    });

    it('Create a new class account', async () => {
        const ix = program.getCreateClassInstruction({
            isPermissioned: false,
            isFrozen: false,
            name: "twitter",
            metadata: "test",
            authority,
            class: classAddress
        })

        console.log("{}", Buffer.from(ix.data).toString("hex"));

        const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

        const transaction = await signTransactionMessageWithSigners(createTransaction({ version: 0, instructions: [ix], feePayer: authority, latestBlockhash }));

        const signature: string = getSignatureFromTransaction(transaction);
        await sendAndConfirmTransaction(transaction);

        console.log(getExplorerLink({ transaction: signature }));
    });
});
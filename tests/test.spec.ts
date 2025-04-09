import * as program from "../clients/js/src/generated/index";
import { createSolanaClient, createTransaction, generateKeyPairSigner, getExplorerLink, getProgramDerivedAddress, getSignatureFromTransaction, KeyPairSigner, signTransactionMessageWithSigners } from "gill";

describe('test', async () => {
    // Generate the keypair first
    const authority: KeyPairSigner = await generateKeyPairSigner();
        
    // Get the address from the public key for PDA derivation
    const name = "twitter";
    const classAddress = (await getProgramDerivedAddress({
        programAddress: program.SRS_PROGRAM_ADDRESS,
        seeds: [
            "class",
            authority.address,
            name
        ]
    }))[0];

    const { rpc, sendAndConfirmTransaction } = createSolanaClient({
        urlOrMoniker: "http://localhost:8899",
    });

    it('Create a new class account', async () => {
        const ix = program.getCreateClassInstruction({
            isPermissioned: false,
            name: "twitter",
            metadata: "test",
            authority,
            class: classAddress
        })

        const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

        const transaction = await signTransactionMessageWithSigners(createTransaction({ version: 0, instructions: [ix], feePayer: authority, latestBlockhash }));

        const signature: string = getSignatureFromTransaction(transaction);
        await sendAndConfirmTransaction(transaction);

        console.log(getExplorerLink({ transaction: signature }));
    });
});
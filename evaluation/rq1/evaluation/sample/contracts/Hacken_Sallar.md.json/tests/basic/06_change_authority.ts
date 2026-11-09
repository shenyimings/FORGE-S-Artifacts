import { Sallar } from "../../target/types/sallar";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { findProgramAddress } from "../utils/pda";
import { PublicKey, Keypair } from '@solana/web3.js';

describe("Sallar - Change Authority", async () => {
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    let new_authority = new Keypair().publicKey;

    const program: Program<Sallar> = anchor.workspace.Sallar;

    let blocks_state_address: anchor.web3.PublicKey = null;
    let _blocks_state_bump: number = null;

    describe("Initializes state", async () => {
        it("Initialize test state", async () => {
            [blocks_state_address, _blocks_state_bump] =
                findProgramAddress("blocks_state");
        });

        it("Pass change Authority", async () => {
            let blocks_state_account = await program.account.blocksState.fetch(
                blocks_state_address,
            );

            assert.equal(
                blocks_state_account.authority.toBase58(),
                provider.wallet.publicKey.toBase58(),
            );

            await program.methods
                .changeAuthority(new_authority)
                .accounts({
                    blocksStateAccount: blocks_state_address,
                    signer: provider.wallet.publicKey,
                })
                .rpc();

            blocks_state_account = await program.account.blocksState.fetch(
                blocks_state_address,
            );

            assert.equal(
                blocks_state_account.authority.toBase58(),
                new_authority.toBase58(),
            );
        });

        it("Fail set change Authority", async () => {
            try {
                await program.methods
                    .changeAuthority(new_authority)
                    .accounts({
                        blocksStateAccount: blocks_state_address,
                        signer: new_authority,
                    })
                    .rpc();

                    assert.fail("Transaction succeeded but was expected to fail");

            } catch (err) {
                assert.equal(
                    err.message,
                    "Signature verification failed",
                );
            }
        });
    });
});

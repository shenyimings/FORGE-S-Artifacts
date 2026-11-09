import { Sallar } from "../../target/types/sallar";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { findProgramAddress } from "../utils/pda";
import { TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { ComputeBudgetProgram, Connection, Transaction } from "@solana/web3.js";
import { getTestAccounts } from "../utils/accounts";

describe("Sallar - top block - user rest mechanism tests", async () => {
	const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);

	const program: Program<Sallar> = anchor.workspace.Sallar;
	const connection = new Connection("http://localhost:8899", "confirmed");

	let blocks_state_address: anchor.web3.PublicKey = null;
	let blocks_state_bump: number = null;

	let authority_address: anchor.web3.PublicKey = null;
	let authority_bump: number = null;

	let mint_address: anchor.web3.PublicKey = null;
	let mint_bump: number = null;

	let distribution_bottom_block_address: anchor.web3.PublicKey = null;
	let distribution_bottom_block_bump: number = null;

	let distribution_top_block_address: anchor.web3.PublicKey = null;
	let distribution_top_block_bump: number = null;

	let final_staking_address: anchor.web3.PublicKey = null;
	let final_staking_account_bump: number = null;

	let final_mining_address: anchor.web3.PublicKey = null;
	let final_mining_account_bump: number = null;

	let testAccounts: Array<anchor.web3.PublicKey> = [];

	let rem_accounts: any = [];

	before(async () => {
		[mint_address, mint_bump] = findProgramAddress("sallar");
		[blocks_state_address, blocks_state_bump] = findProgramAddress("blocks_state");
		[authority_address, authority_bump] = findProgramAddress("authority");
		[final_staking_address, final_staking_account_bump] = findProgramAddress("final_staking");
		[final_mining_address, final_mining_account_bump] = findProgramAddress("final_mining");
		[distribution_top_block_address, distribution_top_block_bump] = findProgramAddress("distribution_top_block");
		[distribution_bottom_block_address, distribution_bottom_block_bump] = findProgramAddress("distribution_bottom_block");

        testAccounts = await getTestAccounts(0,2, connection);
        for (let i = 0; i < testAccounts.length; i++) {
            rem_accounts.push({
                pubkey: testAccounts[i],
                isWritable: true,
                isSigner: false,
            });
        }
	});

    describe("Solve top block", async () => {
        it("FAIL - Additional user request after user request that exceeds available BPs", async () => {
            let userInfoTopBlock = [
                { userPublicKey: testAccounts[0], userRequestWithoutBoost: new anchor.BN(21), userRequestWithBoost: new anchor.BN(21) },
                { userPublicKey: testAccounts[1], userRequestWithoutBoost: new anchor.BN(1), userRequestWithBoost: new anchor.BN(1) }
            ];

            const tx: anchor.web3.Transaction = await program.methods
                .solveTopBlock(userInfoTopBlock as [])
                .remainingAccounts(rem_accounts)
                .accounts({
                    blocksStateAccount: blocks_state_address,
                    distributionTopBlockAccount: distribution_top_block_address,
                    mint: mint_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();
                
                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 1_500_000,
                });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);

            try {
                await provider.sendAndConfirm(transaction, [], {
                    commitment: "processed",
                    maxRetries: 3
                });
                assert.fail("Transaction succeeded but was expected to fail");
            } catch (error) {  
                assert.equal(error.message, 'failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x177a');
                return;
            }
        });

        it("PASS - Single user request that exceeds available BPs processed correctly due to user rest mechanism", async () => {
            let userInfoTopBlock = [
                { userPublicKey: testAccounts[0], userRequestWithoutBoost: new anchor.BN(42), userRequestWithBoost: new anchor.BN(0) }
            ];

            const tx: anchor.web3.Transaction = await program.methods
                .solveTopBlock(userInfoTopBlock as [])
                .remainingAccounts(rem_accounts)
                .accounts({
                    blocksStateAccount: blocks_state_address,
                    distributionTopBlockAccount: distribution_top_block_address,
                    mint: mint_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                })
                .transaction();
            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 1_500_000,
            });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);

            try {
                await provider.sendAndConfirm(transaction, [], {
                    commitment: "processed",
                    maxRetries: 3
                });
            } catch (error) {
                console.log(error);
                assert.fail("Transaction failed but was expected to success.");
            }

            const blocksState = await program.account.blocksState.fetch(blocks_state_address);
            assert.equal(blocksState.topBlockLastAccountAddress.toBase58(), testAccounts[0].toBase58());
            assert.equal(blocksState.topBlockLastAccountRestBp.toNumber(), 22000);
        });
    });
});

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ComputeBudgetProgram, Connection, Transaction } from "@solana/web3.js";
import { assert } from "chai";
import { Sallar } from "../../target/types/sallar";
import { getTestAccounts } from "../utils/accounts";
import { findProgramAddress } from "../utils/pda";

describe("Sallar - Final mining", async () => {
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program: Program<Sallar> = anchor.workspace.Sallar;
    const connection = new Connection("http://localhost:8899", "finalized");

    let blocks_state_address: anchor.web3.PublicKey = null;

    let mint: anchor.web3.PublicKey = null;
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
    let user_info_final_mining = [];

    let organization_account: anchor.web3.PublicKey = null;

    describe("Initializes state", async () => {
        it("Initialize test state", async () => {
            let _blocks_state_bump: number;
            [mint, mint_bump] = findProgramAddress("sallar");
            [blocks_state_address, _blocks_state_bump] =
                findProgramAddress("blocks_state");
            [final_staking_address, final_staking_account_bump] =
                findProgramAddress("final_staking");
            [final_mining_address, final_mining_account_bump] =
                findProgramAddress("final_mining");

            [distribution_top_block_address, distribution_top_block_bump] =
                findProgramAddress("distribution_top_block");
            [
                distribution_bottom_block_address,
                distribution_bottom_block_bump,
            ] = findProgramAddress("distribution_bottom_block");
        });

        it("PASS - Create 24 new remainingTokenAccounts", async () => {
            testAccounts = await getTestAccounts(0, 1, connection);
            organization_account = testAccounts[0];

            for (let i = 0; i < testAccounts.length; i++) {
                rem_accounts.push({
                    pubkey: testAccounts[i],
                    isWritable: true,
                    isSigner: false,
                });

                user_info_final_mining.push({
                    userPublicKey: testAccounts[i],
                    finalMiningBalance: new anchor.BN(12_500_000_000_000),
                });
            }
        });

        describe("Final mining", () => {
            it("FAIL - (Lack Of Funds To Pay The Reward)", async () => {
                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalMining(user_info_final_mining)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalMiningAccount: final_mining_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .transaction();

                    const transaction = new Transaction().add(tx);

                    await provider.sendAndConfirm(transaction, []);

                    assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1777",
                    );

                    return;
                }
            });

            it("Pass - (mining)", async () => {
                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalMining(user_info_final_mining)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalMiningAccount: final_mining_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .transaction();

                    const transaction = new Transaction().add(tx);

                    await provider.sendAndConfirm(transaction, []);

                    assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1777",
                    );
                }
            });

            it("FAIL - (MismatchBetweenRemainingAccountsAndUserInfo)", async () => {
                user_info_final_mining.pop();

                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalMining(user_info_final_mining)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalMiningAccount: final_mining_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .transaction();

                    const additionalComputeBudgetInstruction =
                        ComputeBudgetProgram.requestUnits({
                            units: 384_000,
                            additionalFee: 0,
                        });

                    const transaction = new Transaction()
                        .add(additionalComputeBudgetInstruction)
                        .add(tx);

                    await provider.sendAndConfirm(transaction, []);
                    assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1777",
                    );

                    return;
                }
            });

            it("FAIL - (final Mining feature has not yet been unlocked)", async () => {
                user_info_final_mining.push({
                    userPublicKey: testAccounts[0],
                    finalMiningBalance: new anchor.BN(12_500_000_000_000),
                });
                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalMining(user_info_final_mining)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalMiningAccount: final_mining_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .transaction();

                    const transaction = new Transaction().add(tx);
                    await provider.sendAndConfirm(transaction, []);
                    assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1777",
                    );

                    return;
                }
            });
        });
    });
});

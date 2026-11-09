import { Sallar } from "../../target/types/sallar";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ComputeBudgetProgram, Transaction, Connection } from "@solana/web3.js";
import { getTestAccounts } from "../utils/accounts";
import { describe } from "mocha";
import { findProgramAddress } from "../utils/pda";

describe("Sallar - Final staking", async () => {
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program: Program<Sallar> = anchor.workspace.Sallar;
    const connection = new Connection("http://localhost:8899", "finalized");

    let blocks_state_address: anchor.web3.PublicKey = null;
    let _blocks_state_bump: number = null;

    let mint: anchor.web3.PublicKey = null;
    let mint_bump: number = null;

    let distribution_bottom_block_address: anchor.web3.PublicKey = null;
    let distribution_bottom_block_bump: number = null;

    let distribution_top_block_address: anchor.web3.PublicKey = null;
    let distribution_top_block_bump: number = null;

    let final_staking_address: anchor.web3.PublicKey = null;
    let final_staking_account_bump: number = null;

    let testAccounts: Array<anchor.web3.PublicKey> = [];

    let rem_accounts: any = [];
    let user_info_final_staking = [];

    describe("Initializes state", async () => {
        it("Initialize test state", async () => {
            [mint, mint_bump] = findProgramAddress("sallar");
            [blocks_state_address, _blocks_state_bump] =
                findProgramAddress("blocks_state");
            [final_staking_address, final_staking_account_bump] =
                findProgramAddress("final_staking");
            [distribution_top_block_address, distribution_top_block_bump] =
                findProgramAddress("distribution_top_block");
            [
                distribution_bottom_block_address,
                distribution_bottom_block_bump,
            ] = findProgramAddress("distribution_bottom_block");
        });

        it("PASS - Create new remainingTokenAccounts", async () => {
            testAccounts = await getTestAccounts(0, 1, connection);

            for (let i = 0; i < testAccounts.length; i++) {
                rem_accounts.push({
                    pubkey: testAccounts[i],
                    isWritable: true,
                    isSigner: false,
                });

                user_info_final_staking.push({
                    userPublicKey: testAccounts[i],
                    rewardPart: 0.01,
                });
            }
        });

        describe("State", () => {
            it("FAIL - (Final Staking Pool In Round Is Empty.)", async () => {
                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
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

            it("FAIL - (final_staking) before solve all blocks", async () => {
                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
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

            it("FAIL - (UserRequestExceedsAvailableRewardParts)", async () => {
                user_info_final_staking.pop();
                user_info_final_staking.push({
                    userPublicKey: testAccounts[0],
                    rewardPart: -1.0,
                });

                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
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

            it("FAIL - (UserRewardPartsSumTooHigh)", async () => {
                user_info_final_staking.pop();
                user_info_final_staking.push({
                    userPublicKey: testAccounts[0],
                    rewardPart: 1.01,
                });

                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
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

            it("FAIL - (MismatchBetweenRemainingAccountsAndUserInfo)", async () => {
                user_info_final_staking.pop();

                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
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

                    await provider.sendAndConfirm(
                        transaction,
                        [],
                    );

                    assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1777",
                    );

                    return;
                }
            });

            it("PASS - Create new remainingTokenAccounts", async () => {
                testAccounts = [];
                testAccounts = await getTestAccounts(0, 1, connection);

                for (let i = 0; i < testAccounts.length; i++) {
                    rem_accounts.push({
                        pubkey: testAccounts[i],
                        isWritable: true,
                        isSigner: false,
                    });
                    user_info_final_staking = [];
                    user_info_final_staking.push({
                        userPublicKey: testAccounts[i],
                        rewardPart: 0.8,
                    });
                }
            });

            it("FAIL - (final_staking_account feature has not yet been unlocked)", async () => {
                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .transaction();

                    const transaction = new Transaction().add(tx);

                    await provider.sendAndConfirm(
                        transaction,
                        [],
                    );
                    assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1777",
                    );

                    return;
                }
            });

            it("FAIL - (Wrong blocks_state_address)", async () => {
                try {
                    const invalid_blocks_state_address: anchor.web3.Keypair =
                        anchor.web3.Keypair.generate();

                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount:
                                invalid_blocks_state_address.publicKey,
                            finalStakingAccount: final_staking_address,
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
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 0: custom program error: 0xbc4",
                    );
                    return;
                }
            });

            it("FAIL - (Wrong final_staking_account)", async () => {
                const invalid_TokenInstructions: anchor.web3.Keypair =
                    anchor.web3.Keypair.generate();

                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
                            tokenProgram: invalid_TokenInstructions.publicKey,
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
                        "failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0xbc0",
                    );
                    return;
                }
            });

            it("FAIL - (Wrong final_staking_account)", async () => {
                const invalid_signer: anchor.web3.Keypair =
                    anchor.web3.Keypair.generate();

                try {
                    const tx: anchor.web3.Transaction = await program.methods
                        .finalStaking(user_info_final_staking)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            finalStakingAccount: final_staking_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: invalid_signer.publicKey,
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
                        "Signature verification failed",
                    );
                    return;
                }
            });
        });
    });
});

import { Sallar } from "../../target/types/sallar";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";
import { findProgramAddress } from "../utils/pda";
import { ComputeBudgetProgram, Connection, Transaction } from "@solana/web3.js";
import { getTestAccounts } from "../utils/accounts";

describe("Sallar - bottom block", async () => {
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program: Program<Sallar> = anchor.workspace.Sallar;
    const connection = new Connection("http://localhost:8899", "confirmed");

    let blocks_state_address: anchor.web3.PublicKey = null;
    let blocks_state_bump: number = null;

    let authority_address: anchor.web3.PublicKey = null;
    let authority_bump: number = null;

    let mint_address: anchor.web3.PublicKey = null;

    let distribution_bottom_block_address: anchor.web3.PublicKey = null;

    let testAccounts: Array<anchor.web3.PublicKey> = [];

    let rem_accounts: any = [];
    let user_info_bottom_block = [];
    let user_info_final_staking = [];

    describe("Initializes state", async () => {
        it("Initialize test state", async () => {
            let _mint_bump: number;
            let _distribution_bottom_block_bump: number;
            [mint_address, _mint_bump] = findProgramAddress("sallar");
            [blocks_state_address, blocks_state_bump] =
                findProgramAddress("blocks_state");
            [
                distribution_bottom_block_address,
                _distribution_bottom_block_bump,
            ] = findProgramAddress("distribution_bottom_block");
            [authority_address, authority_bump] =
                findProgramAddress("authority");
        });

        it("PASS - Create 2 new remainingTokenAccounts", async () => {
            testAccounts = await getTestAccounts(0, 2, connection);
            for (let i = 0; i < testAccounts.length; i++) {
                rem_accounts.push({
                    pubkey: testAccounts[i],
                    isWritable: true,
                    isSigner: false,
                });

                user_info_bottom_block.push({
                    userPublicKey: testAccounts[i],
                    userBalance: new anchor.BN(2_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(1),
                    userRequestWithBoost: new anchor.BN(0),
                });
                user_info_final_staking.push({
                    userPublicKey: testAccounts[i],
                    rewardPart: 0.2,
                });
            }
        });

        describe("Solve bottom block", async () => {
            it("PASS - Success solve bottom block", async () => {
                const tx: anchor.web3.Transaction = await program.methods
                    .solveBottomBlock(user_info_bottom_block)
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        blocksStateAccount: blocks_state_address,
                        distributionBottomBlockAccount:
                            distribution_bottom_block_address,
                        mint: mint_address,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .transaction();

                const additionalComputeBudgetInstruction =
                    ComputeBudgetProgram.setComputeUnitLimit({
                        units: 500_000,
                    });
                const transaction = new Transaction()
                    .add(additionalComputeBudgetInstruction)
                    .add(tx);

                await provider.sendAndConfirm(transaction, [], {
                    commitment: "confirmed",
                    maxRetries: 3,
                });
            });

            it("PASS - checking the mechanism of automatic token minting", async () => {
                for (let i = 0; i < 25; i++) {
                    const tx: anchor.web3.Transaction = await program.methods
                        .solveBottomBlock(user_info_bottom_block)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionBottomBlockAccount:
                                distribution_bottom_block_address,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .transaction();

                    const additionalComputeBudgetInstruction =
                        ComputeBudgetProgram.setComputeUnitLimit({
                            units: 500_000,
                        });
                    const transaction = new Transaction()
                        .add(additionalComputeBudgetInstruction)
                        .add(tx);

                    await provider.sendAndConfirm(transaction, [], {
                        commitment: "confirmed",
                        maxRetries: 3,
                    });
                }

                const blocksState = await program.account.blocksState.fetch(
                    blocks_state_address,
                );
            });

            it("FAIL - (Wrong blocks_state_address)", async () => {
                const invalid_blocks_state_address: anchor.web3.Keypair =
                    anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveBottomBlock(user_info_bottom_block)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount:
                                invalid_blocks_state_address.publicKey,
                            distributionBottomBlockAccount:
                                distribution_bottom_block_address,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "Signature verification failed",
                    );
                    return;
                }
            });

            it("FAIL - (Wrong distribution_bottom_block_address)", async () => {
                const invalid_distribution_bottom_block_address: anchor.web3.Keypair =
                    anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveBottomBlock(user_info_bottom_block)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionBottomBlockAccount:
                                invalid_distribution_bottom_block_address.publicKey,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "Signature verification failed",
                    );
                    return;
                }
            });

            it("FAIL - (Wrong Mint)", async () => {
                const invalid_mint: anchor.web3.Keypair =
                    anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveBottomBlock(user_info_bottom_block)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionBottomBlockAccount:
                                distribution_bottom_block_address,
                            mint: invalid_mint.publicKey,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "Signature verification failed",
                    );
                    return;
                }
            });

            it("FAIL - (Wrong tokenProgram)", async () => {
                const invalid_tokenProgram: anchor.web3.Keypair =
                    anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveBottomBlock(user_info_bottom_block)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionBottomBlockAccount:
                                distribution_bottom_block_address,
                            mint: mint_address,
                            tokenProgram: invalid_tokenProgram.publicKey,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "Signature verification failed",
                    );
                    return;
                }
            });

            it("FAIL - (Wrong Authority)", async () => {
                const invalid_authority: anchor.web3.Keypair =
                    anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveBottomBlock(user_info_bottom_block)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionBottomBlockAccount:
                                distribution_bottom_block_address,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: invalid_authority.publicKey,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "Signature verification failed",
                    );

                    return;
                }
            });

            it("FAIL - (Blocks collision)", async () => {
                try {
                    const tx: string = await program.methods
                        .solveBottomBlock(user_info_bottom_block)
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionBottomBlockAccount:
                                distribution_bottom_block_address,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.message,
                        "Signature verification failed",
                    );
                    return;
                }
            });

            it("PASS - User request exceeds available BPs", async () => {
                user_info_bottom_block.pop();
                user_info_bottom_block.pop();
                user_info_bottom_block.push({
                    userPublicKey: testAccounts[0],
                    userBalance: new anchor.BN(200_000_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(42),
                    userRequestWithBoost: new anchor.BN(42),
                });
                user_info_bottom_block.push({
                    userPublicKey: testAccounts[1],
                    userBalance: new anchor.BN(200_000_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(42),
                    userRequestWithBoost: new anchor.BN(42),
                });
                const tx: anchor.web3.Transaction = await program.methods
                    .solveBottomBlock(user_info_bottom_block)
                    .remainingAccounts(rem_accounts)
                    .accounts({
                        blocksStateAccount: blocks_state_address,
                        distributionBottomBlockAccount:
                            distribution_bottom_block_address,
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

                await provider.sendAndConfirm(transaction, [], {
                    commitment: "confirmed",
                    maxRetries: 3,
                });
            });
        });
    });
});

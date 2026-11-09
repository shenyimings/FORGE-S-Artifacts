import { Sallar } from "../../target/types/sallar";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { findProgramAddress } from "../utils/pda";
import { TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { ComputeBudgetProgram, Connection, Transaction } from "@solana/web3.js";
import { getTestAccounts } from "../utils/accounts";

describe("Sallar - top block", async () => {
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
	let user_info_top_block = [];
	let user_info_bottom_block = [];
	let user_info_final_staking = [];

	let organization_account: anchor.web3.PublicKey = null;

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

            user_info_top_block.push({ userPublicKey: testAccounts[i], userRequestWithoutBoost: new anchor.BN(1), userRequestWithBoost: new anchor.BN(0) });
            user_info_bottom_block.push({ userPublicKey: testAccounts[i], userBalance: new anchor.BN(20_000_000_000), userRequestWithoutBoost: new anchor.BN(1), userRequestWithBoost: new anchor.BN(0) });
            user_info_final_staking.push({ userPublicKey: testAccounts[i], rewardPart: 0.20 });
        }
	});

        describe("Solve top block", async () => {
            it("PASS - Success first solve", async () => {
                const tx: anchor.web3.Transaction = await program.methods
                    .solveTopBlock(user_info_top_block)
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
        
                await provider.sendAndConfirm(transaction, [], {
                    commitment: "confirmed",
                    maxRetries: 3
                });
            });

            it("PASS - checking the mechanism of automatic token minting", async () => {	
                for (let i = 0; i < 5; i++) {
                    const tx: anchor.web3.Transaction = await program.methods
                    .solveTopBlock(user_info_top_block)
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
                        units: 520_000,
                    });
                    const transaction = new Transaction()
                        .add(additionalComputeBudgetInstruction)
                        .add(tx);
        
                    await provider.sendAndConfirm(transaction, [], {
                        commitment: "confirmed",
                        maxRetries: 3
                    });
                }
            });

            it("FAIL - (Wrong blocks_state_address)", async () => {
                const invalid_blocks_state_address: anchor.web3.Keypair = anchor.web3.Keypair.generate();
                try {
                    const tx: string = await program.methods
                        .solveTopBlock(
                            user_info_top_block
                        )
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: invalid_blocks_state_address.publicKey,
                            distributionTopBlockAccount: distribution_top_block_address,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(error.message, 'Signature verification failed');
                    return;
                }
            });

            it("FAIL - (Wrong distribution_top_block_address)", async () => {
                const invalid_distribution_top_block_address: anchor.web3.Keypair = anchor.web3.Keypair.generate();
                try {
                    const tx: string = await program.methods
                        .solveTopBlock(
                            user_info_top_block
                        )
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionTopBlockAccount: invalid_distribution_top_block_address.publicKey,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(error.message, 'Signature verification failed');
                    return;
                }
            });

            it("FAIL - (Wrong Mint)", async () => {
                const invalid_mint: anchor.web3.Keypair = anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveTopBlock(
                            user_info_top_block
                        )
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionTopBlockAccount: distribution_top_block_address,
                            mint: invalid_mint.publicKey,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(error.message, 'Signature verification failed');
                
                    return;
                }
            });

            it("FAIL - (Wrong tokenProgram)", async () => {
                const invalid_tokenProgram: anchor.web3.Keypair = anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveTopBlock(
                            user_info_top_block
                        )
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionTopBlockAccount: distribution_top_block_address,
                            mint: mint_address,
                            tokenProgram: invalid_tokenProgram.publicKey,
                            signer: authority_address,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(error.message, 'Signature verification failed');
                
                    return;
                }
            });

            it("FAIL - (Wrong Authority)", async () => {
                const invalid_authority: anchor.web3.Keypair = anchor.web3.Keypair.generate();

                try {
                    const tx: string = await program.methods
                        .solveTopBlock(
                            user_info_top_block
                        )
                        .remainingAccounts(rem_accounts)
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            distributionTopBlockAccount: distribution_top_block_address,
                            mint: mint_address,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: invalid_authority.publicKey,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(error.message, 'Signature verification failed');
                    
                    return;
                }
            });

            it("FAIL - (Blocks collision)", async () => {
				try {
					const tx: string = await program.methods
						.solveTopBlock(
							user_info_top_block
						)
						.remainingAccounts(rem_accounts)
						.accounts({
							blocksStateAccount: blocks_state_address,
							distributionTopBlockAccount: distribution_top_block_address,
							mint: mint_address,
							tokenProgram: TOKEN_PROGRAM_ID,
							signer: authority_address,
						})
						.rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
				} catch (error) {
					assert.equal(error.message, 'Signature verification failed');
					
					return;
				}
			});
        });
    });

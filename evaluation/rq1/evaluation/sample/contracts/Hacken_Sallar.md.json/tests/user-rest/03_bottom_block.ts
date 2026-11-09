import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ComputeBudgetProgram, Connection, Transaction } from "@solana/web3.js";
import { Sallar } from "../../target/types/sallar";
import { findProgramAddress } from "../utils/pda";
import { assert } from "chai";
import { getTestAccounts } from "../utils/accounts";

describe("Sallar - bottom block - user rest mechanism", async () => {
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

    let rem_accounts: any = [];
    let user_info_bottom_block = [];

    describe("Initializes state", async() => {
        before(async () => {
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

        it("FAIL - Additional user request after user request that exceeds available BPs", async () => {
            let testAccounts: Array<anchor.web3.PublicKey> = await getTestAccounts(0, 3, connection);
            for (let i = 0; i < testAccounts.length; i++) {
                rem_accounts.push({
                    pubkey: testAccounts[i],
                    isWritable: true,
                    isSigner: false,
                });
            }
            user_info_bottom_block = [
                {
                    userPublicKey: testAccounts[0],
                    userBalance: new anchor.BN(200_000_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(255),
                    userRequestWithBoost: new anchor.BN(255)
                },
                {
                    userPublicKey: testAccounts[1],
                    userBalance: new anchor.BN(200_000_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(255),
                    userRequestWithBoost: new anchor.BN(255),
                },
                {
                    userPublicKey: testAccounts[2],
                    userBalance: new anchor.BN(200_000_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(255),
                    userRequestWithBoost: new anchor.BN(255),
                }
            ];
            const tx: anchor.web3.Transaction = await program.methods
                .solveBottomBlock(user_info_bottom_block)
                .remainingAccounts(rem_accounts)
                .accounts({
                    blocksStateAccount: blocks_state_address,
                    distributionBottomBlockAccount: distribution_bottom_block_address,
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

        it("PASS - Two user requests where second exceeds available BPs are processed correctly due to user rest mechanism", async () => {
            let testAccounts: Array<anchor.web3.PublicKey> = await getTestAccounts(0, 2, connection);
            for (let i = 0; i < testAccounts.length; i++) {
                rem_accounts.push({
                    pubkey: testAccounts[i],
                    isWritable: true,
                    isSigner: false,
                });
            }
            user_info_bottom_block = [
                {
                    userPublicKey: testAccounts[0],
                    userBalance: new anchor.BN(200_000_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(255),
                    userRequestWithBoost: new anchor.BN(255)
                },
                {
                    userPublicKey: testAccounts[1],
                    userBalance: new anchor.BN(200_000_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(255),
                    userRequestWithBoost: new anchor.BN(255),
                }
            ];
            const tx: anchor.web3.Transaction = await program.methods
                .solveBottomBlock(user_info_bottom_block)
                .remainingAccounts(rem_accounts)
                .accounts({
                    blocksStateAccount: blocks_state_address,
                    distributionBottomBlockAccount: distribution_bottom_block_address,
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
            assert.equal(blocksState.bottomBlockLastAccountAddress.toBase58(), testAccounts[testAccounts.length - 1].toBase58());
            assert.equal(blocksState.bottomBlockLastAccountRestBp.toNumber(), 9363148050);
        }); 
    });
});

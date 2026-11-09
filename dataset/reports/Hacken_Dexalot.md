## Audit Scope

Scope Details

Repository	https://github.com/Dexalot/contracts
Initial Commit	09afb2bbb15fe9d36a180152149d8aa056d99db8

We express our gratitude to the Dexalot team for the collaborative engagement that enabled the execution of this Smart Contract Security Assessment.

Dexalot is a revolutionary decentralized exchange that features an on-chain Central Limit Order Book (CLOB), providing a trading experience similar to traditional centralized exchanges. Operating on its own dedicated Avalanche subnet and expanding cross-chain, Dexalot offers enhanced trading efficiency and accessibility across multiple blockchain ecosystems.

## F-2025-10738 Arbitrary Funds Destination due to Lack of Accounts Validation in LzReceive

Severity: High

Description
According to LayerZero implementation →, the cross-chain message is loaded to the destination chain via the InitVerify instruction. According to the documentation →, Executor (or any third party) is able to invoke the LzReceive instruction in the contract, the contract is responsible for validating the provided data matches the data recorded in the LayerZero smart contract via invoking the Clear instruction.

The LzReceive instruction implementation does not validate the provided accounts match the cross-chain message allowing the Executor (or any third party) provide arbitrary receiver address for the funds withdrawal.

pub struct LzReceive<'info> {
    ...
    /// CHECK: ata or trader
    #[account(mut)]
    pub to: AccountInfo<'info>,
    ...
}
This may result in third-party executing LzReceive prior to fair Executor and withdraw funds to arbitrary destination.

## F-2025-10107 Pause Functionality Implemented Partially

Severity: Medium

Description
The global_config.program_paused flag is responsible for pausing operation on the program.

While Deposit, DepositNative, DepositAirdrop, LzReceive, RemoveToken, CrossSwap, and Swap instructions are paused, some other instructions such as AddToken, RemoveFromSwapQueue, FundSpl, FundSol, ClaimNativeBalance, ClaimSplBalance, UpdateSwapExpiry are still accessible.

The behavior is considered to be inconsistent and might be a sign of unfinalized code.

Inconsistent pausing functionality may lead to inability to pause the protocol in a critical situation.

## F-2025-10102 Arbitrary LayerZero Endpoint Program

Severity: Medium

Description
The Initialize, CrossSwap, Deposit, DepositNative instructions perform calls to LayerZero Endpoint program. The Endpoint program address is not validated allowing arbitrary program to be executed.

#[derive(Accounts)]
pub struct CrossSwap<'info> {
    ...

    pub endpoint_program: AccountInfo<'info>,
}

pub fn send_cross_chain_trade(&self, ctx: &mut Context<CrossSwap>) -> Result<()> {
    ...

    let send_remaining_accounts = &ctx.remaining_accounts[QUOTE_REMAINING_ACCOUNTS_COUNT..];
    let send_params = EndpointSendParams {
        dst_eid: self.dest_chain_id as u32,
        receiver: remote.address,
        message,
        options: GAS_OPTIONS.to_vec(),
        native_fee: fee.native_fee,
        lz_token_fee: 0,
    };

    let send_seeds: &[&[&[u8]]] = &[&[PORTFOLIO_SEED, &[ctx.accounts.portfolio.bump]]];

    let send_cpi_data = create_instruction_data(&send_params, ENDPOINT_SEND)?;

    let portfolio_key = ctx.accounts.portfolio.key();
    let send_accounts_metas: Vec<AccountMeta> = send_remaining_accounts
        .iter()
        .skip(1) // an account is skipped because we don't use layerzero cpi utils so it's not needed
        .map(|account| AccountMeta {
            pubkey: *account.key,
            is_signer: account.key() == portfolio_key || account.is_signer,
            is_writable: account.is_writable,
        })
        .collect();

    invoke_signed(
        &Instruction {
            program_id: endpoint_program.key(),
            accounts: send_accounts_metas,
            data: send_cpi_data,
        },
        send_remaining_accounts,
        send_seeds,
    )?;

    ...
}

While arbitrary program execution does not break the program consistency, the possibility poses a risks for users mistakenly signed transaction with a malicious program provided instead of valid LayerZero Endpoint. User account is passed as signer to the program what means the program receives access to any user funds.

## F-2025-10100 Supported Tokens List Functionality Implemented Partially

Severity: Medium

Description
The program state structure includes TokenList PDA. The PDA seeds are [TOKEN_LIST_SEED, page].

#[account]
pub struct TokenList {
    pub next_page: Option<Pubkey>,
    #[max_len(MAX_TOKENS)] // MAX_TOKENS = 20
    pub tokens: Vec<Pubkey>,
}
From the above it implies that list of supported tokens may exceed 20 units and go over several PDAs. However, most of the program instructions ignores this or provides partial support.

The LzReceive instruction processes only the first page in the tokens list, aborting if the target token is not included there.

#[derive(Accounts)]
pub struct CrossSwap<'info> {
    ...

    pub endpoint_program: AccountInfo<'info>,
}

pub fn send_cross_chain_trade(&self, ctx: &mut Context<CrossSwap>) -> Result<()> {
    ...

    let send_remaining_accounts = &ctx.remaining_accounts[QUOTE_REMAINING_ACCOUNTS_COUNT..];
    let send_params = EndpointSendParams {
        dst_eid: self.dest_chain_id as u32,
        receiver: remote.address,
        message,
        options: GAS_OPTIONS.to_vec(),
        native_fee: fee.native_fee,
        lz_token_fee: 0,
    };

    let send_seeds: &[&[&[u8]]] = &[&[PORTFOLIO_SEED, &[ctx.accounts.portfolio.bump]]];

    let send_cpi_data = create_instruction_data(&send_params, ENDPOINT_SEND)?;

    let portfolio_key = ctx.accounts.portfolio.key();
    let send_accounts_metas: Vec<AccountMeta> = send_remaining_accounts
        .iter()
        .skip(1) // an account is skipped because we don't use layerzero cpi utils so it's not needed
        .map(|account| AccountMeta {
            pubkey: *account.key,
            is_signer: account.key() == portfolio_key || account.is_signer,
            is_writable: account.is_writable,
        })
        .collect();

    invoke_signed(
        &Instruction {
            program_id: endpoint_program.key(),
            accounts: send_accounts_metas,
            data: send_cpi_data,
        },
        send_remaining_accounts,
        send_seeds,
    )?;

    ...
}
The AddToken instruction has an algorithm implemented to loop over several tokens list pages, however, it never reaches the second page as this requires next_page field to be initialized while it is not initialized when the operating page is filled fully.

pub fn add_token<'info>(
    ctx: &mut Context<'_, '_, 'info, 'info, AddToken<'info>>,
    params: &AddTokenParams,
) -> Result<()> {
    ...

    while let Some(token_list_info) = iter.next() {
        let mut token_list = Account::<TokenList>::try_from(token_list_info)?;
        let token_mint = params.token_address;
        ...
        if token_list.tokens.len() < MAX_TOKENS {
            // token_list.next_page is not set if MAX_TOKENS reached
            token_list.tokens.push(token_mint);
            token_list.exit(&crate::ID)?;
            break;
        } else if token_list.next_page.is_some() && iter.peek().is_some() {
            // Next cycle iteration happens only if token_list.next_page is set
            continue;
        } else {
            return Err(DexalotError::TokenListFull.into());
        }
    }

    ...
}
Due to the mentioned limitations, the system allows MAX_TOKENS = 20 supported tokens at maximum. The functionality is not scalable and requires a redesign to support unlimited amount of tokens.

## F-2025-10108 Redundant Bumps Recalculation

Severity
Low

Description
The Protfolio PDA data includes bump field containing the bump seed.

#[account]
pub struct Portfolio {
    pub admin: Pubkey,
    pub global_config: GlobalConfig,
    pub bump: u8,
}
The field can be used to reduce Gas wastes during the PDA check. The optimization is implemented in the LzReceive instruction and is missing for the Deposit, DepositNative, DepositAirdrop, GetGlobalConfig, WriteConfig, SetRemote, RemoveToken, CrossSwap, Swap instructions.

// Cached
#[account(seeds = [PORTFOLIO_SEED], bump = portfolio.bump)]
pub portfolio: Account<'info, Portfolio>,

// Recalculated each time
#[account(seeds = [PORTFOLIO_SEED], bump)]
pub portfolio: Account<'info, Portfolio>,
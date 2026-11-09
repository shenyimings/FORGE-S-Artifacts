use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_token_2022::extension::StateWithExtensions;
use super_lendy::state::reserve::LpExchangeRate;
use super_lendy::{state::curator::Curator, LendyResult};
use texture_common::error::InvalidKey;
use texture_common::math::{CheckedAdd, CheckedDiv, Decimal};
use texture_common::{account::PodAccount, utils::verify_key};
use tinyvec::ArrayVec;

use crate::{
    error::VaultError, instruction::VaultInstruction, state::vault::Vault, VaultResult, VAULT_ID,
};

mod position;
mod rewards;
mod vault;

pub type SeedVec<'a> = ArrayVec<[&'a [u8]; 5]>;
macro_rules! seedvec {
    ($($seed:expr),*) => {{
        let mut seed_vec = $crate::processor::SeedVec::new();
        $( seed_vec.push($seed); )*
        seed_vec
    }};
}
use crate::pda::find_program_authority;
pub(crate) use seedvec;

pub struct Processor<'a, 'b> {
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'b>],
}

impl<'a, 'b> Processor<'a, 'b> {
    pub fn new(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'b>]) -> Self {
        Self {
            program_id,
            accounts,
        }
    }
    pub fn process_instruction(&self, input: &[u8]) -> VaultResult<()> {
        // Two process functions - is to overcome stack size limitation for IX arguments

        match self.process_instruction_1(input) {
            Ok(result) => Ok(result),
            Err(VaultError::Unimplemented) => self.process_instruction_2(input),
            Err(err) => Err(err),
        }
    }

    pub fn process_instruction_1(&self, input: &[u8]) -> VaultResult<()> {
        match VaultInstruction::try_from_slice(input).map_err(VaultError::from)? {
            VaultInstruction::Version { print_version } => self.version(print_version),
            VaultInstruction::CreateVault {
                params,
                reserves_count,
                lp_token_mints_count,
                lp_token_storages_count,
            } => self.create_vault(
                params,
                reserves_count,
                lp_token_mints_count,
                lp_token_storages_count,
            ),
            VaultInstruction::AlterVault { params } => self.alter_vault(params),
            VaultInstruction::ProposeDistributionItem { index } => {
                self.propose_distribution_item(index)
            }
            VaultInstruction::AddDistributionItem { index } => self.add_distribution_item(index),
            VaultInstruction::RemoveDistributionItem { reserve } => {
                self.remove_distribution_item(reserve)
            }
            VaultInstruction::CreatePosition => self.create_position(),
            VaultInstruction::Deposit {
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
                amount,
            } => self.deposit(
                amount,
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
            ),
            VaultInstruction::Withdraw {
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
                vlp_amount,
            } => self.withdraw(
                vlp_amount,
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
            ),
            VaultInstruction::ClosePosition => self.close_position(),
            VaultInstruction::Rebalance {
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
            } => self.rebalance(
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
            ),
            _ => Err(VaultError::Unimplemented),
        }
    }

    pub fn process_instruction_2(&self, input: &[u8]) -> VaultResult<()> {
        match VaultInstruction::try_from_slice(input).map_err(VaultError::from)? {
            VaultInstruction::SetRewardRules { rules } => self.set_reward_rules(rules),
            VaultInstruction::InitRewardSupply => self.init_reward_supply(),
            VaultInstruction::ClaimReward => self.claim_reward(),
            VaultInstruction::WithdrawReward { amount } => self.withdraw_reward(amount),
            VaultInstruction::ClaimFees {
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
                min_fee_amount,
            } => self.claim_fees(
                reserves_count,
                reserve_liquidity_supplies_count,
                lp_token_mints_count,
                lp_token_storages_count,
                min_fee_amount,
            ),
            _ => {
                msg!("Unrecognized IX");
                Err(VaultError::Unimplemented)
            }
        }
    }

    #[inline(never)]
    pub(super) fn version(&self, print_version: bool) -> VaultResult<()> {
        if print_version {
            msg!("vault contract {}", env!("CARGO_PKG_VERSION").to_string());
            return Err(VaultError::OperationCanNotBePerformed);
        }
        Ok(())
    }
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &VAULT_ID {
        msg!("IX in not for Vault but for {}", program_id);
        return Err(ProgramError::IncorrectProgramId);
    }

    Processor::new(program_id, accounts)
        .process_instruction(instruction_data)
        .map_err(|err| {
            msg!("Error: {}", err);
            err.into()
        })
}

/// Checks that Curator account as in the Vault and checks that curator_vaults_authority
/// is correct.
pub fn verify_curator(
    vault: &AccountInfo<'_>,
    curator: &AccountInfo<'_>,
    curator_vaults_authority: &AccountInfo<'_>,
) -> LendyResult<()> {
    let vault_data = vault.data.borrow();
    let unpacked_vault = Vault::try_from_bytes(&vault_data)?;

    let curator_data = curator.data.borrow();
    let unpacked_curator = Curator::try_from_bytes(&curator_data)?;

    verify_key(&unpacked_vault.curator, curator.key, "curator vs vault")?;
    verify_key(
        curator_vaults_authority.key,
        &unpacked_curator.vaults_authority,
        "curator_vaults_authority vs curator.vaults_authority",
    )?;

    Ok(())
}

/// Transfers `amount` lamports from `from_account` (must be program owned)
/// to another `to_account`. The `to_account` can be owned by anyone else.
pub fn transfer_lamports(
    from_account: &AccountInfo<'_>,
    to_account: &AccountInfo<'_>,
    amount: u64,
) -> VaultResult<()> {
    if **from_account
        .try_borrow_lamports()
        .map_err(|_| VaultError::OperationCanNotBePerformed)?
        < amount
    {
        return Err(VaultError::OperationCanNotBePerformed);
    }

    **from_account
        .try_borrow_mut_lamports()
        .map_err(|_| VaultError::OperationCanNotBePerformed)? -= amount;
    **to_account
        .try_borrow_mut_lamports()
        .map_err(|_| VaultError::OperationCanNotBePerformed)? += amount;

    msg!(
        "transfer_lamports {} from {} to {}",
        amount,
        from_account.key,
        to_account.key
    );

    Ok(())
}

// Checks that 4 counts are the same
pub fn verify_account_count(cnt0: u8, cnt1: u8, cnt2: u8, cnt3: u8) -> VaultResult<()> {
    for actual_count in [cnt0, cnt1, cnt2] {
        if cnt3 != actual_count {
            return Err(VaultError::InvalidAccountsCount);
        }
    }

    Ok(())
}

pub fn verify_program_authority(provided_authority: &Pubkey) -> VaultResult<u8> {
    let (expected_authority, authority_bump) = find_program_authority();
    verify_key(provided_authority, &expected_authority, "program authority")?;

    Ok(authority_bump)
}

/// Calculate price of 1 VLP in Vault liquidity currency
pub fn vlp_price(
    lp_exchange_rates: &[LpExchangeRate],
    lp_amounts: &[Decimal],
    vlp_supply: Decimal,
) -> Result<Decimal, VaultError> {
    if vlp_supply == Decimal::ZERO {
        return Ok(Decimal::ONE);
    }
    let total_liquidity_supply = total_liquidity_supply(lp_amounts, lp_exchange_rates)?;
    let vlp_price = total_liquidity_supply.checked_div(vlp_supply)?;
    Ok(vlp_price)
}

/// Calculate total liquidity supply of Vault
pub fn total_liquidity_supply(
    lp_amounts: &[Decimal],
    lp_exchange_rates: &[LpExchangeRate],
) -> Result<Decimal, VaultError> {
    Ok(lp_amounts.iter().zip(lp_exchange_rates).try_fold(
        Decimal::ZERO,
        |acc, (lp_amount, lp_exchange_rate)| {
            let liquidity_amount = lp_exchange_rate.decimal_lp_to_liquidity(*lp_amount)?;
            acc.checked_add(liquidity_amount)
        },
    )?)
}

/// Vault can work only with two predefined Token programs
pub fn verify_token_program(token_program: &AccountInfo<'_>) -> VaultResult<()> {
    if token_program.key != &spl_token::id() && token_program.key != &spl_token_2022::id() {
        Err(InvalidKey {
            key_type: "Token program",
            actual: *token_program.key,
            expected: spl_token::id(),
        })?
    } else {
        Ok(())
    }
}

/// Unpacks Mint account of either Token or Token2022 and returns it's `decimals` field
pub fn mint_decimals(mint: &AccountInfo<'_>) -> VaultResult<u8> {
    if mint.owner == &spl_token::id() {
        let unpacked_liquidity_mint = spl_token::state::Mint::unpack(&mint.data.borrow())
            .map_err(|err| VaultError::AccountUnpackError(*mint.key, err))?;
        Ok(unpacked_liquidity_mint.decimals)
    } else if mint.owner == &spl_token_2022::id() {
        let mint_data = mint.data.borrow();
        let unpacked_liquidity_mint =
            StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)
                .map_err(|err| VaultError::AccountUnpackError(*mint.key, err))?;
        Ok(unpacked_liquidity_mint.base.decimals)
    } else {
        Err(InvalidKey {
            key_type: "Token program",
            actual: *mint.owner,
            expected: spl_token::id(),
        })?
    }
}

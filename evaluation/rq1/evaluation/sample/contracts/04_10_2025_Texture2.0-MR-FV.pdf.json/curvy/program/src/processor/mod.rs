use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use texture_common::account::PodAccount;
use texture_common::remote::system::SystemProgram;
use texture_common::utils::verify_key;

use crate::error::CurvyError;
use crate::instruction::{
    AlterCurveAccounts, CreateCurveAccounts, CurvyInstruction, DeleteCurveAccounts,
};
use crate::state::curve::{Curve, CurveParams};
use crate::CurvyResult;

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

    pub fn process_instruction(self, input: &[u8]) -> CurvyResult<()> {
        match CurvyInstruction::try_from_slice(input).map_err(CurvyError::from)? {
            CurvyInstruction::CreateCurve { params } => self.create_curve(params),
            CurvyInstruction::AlterCurve { params } => self.alter_curve(params),
            CurvyInstruction::DeleteCurve => self.delete_curve(),
        }
    }

    #[inline(never)]
    pub(super) fn create_curve(self, params: CurveParams) -> CurvyResult<()> {
        msg!("create_curve ix");

        let CreateCurveAccounts {
            curve,
            owner,
            system_program,
        } = CreateCurveAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let rent = Rent::get().expect("No Rent");

        SystemProgram::new(system_program)
            .create_account(
                owner,
                curve,
                Curve::SIZE as u64,
                rent.minimum_balance(Curve::SIZE),
                self.program_id,
            )
            .call()?;

        Curve::check_params(&params)?;

        let mut curve_data = curve.data.borrow_mut();

        Curve::init_bytes(&mut curve_data, (params, *owner.key))?;

        Ok(())
    }

    #[inline(never)]
    fn alter_curve(&self, params: CurveParams) -> Result<(), CurvyError> {
        msg!("alter_curve ix");

        let AlterCurveAccounts { curve, owner } =
            AlterCurveAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let mut curve_data = curve.data.borrow_mut();
        let curve = Curve::try_from_bytes_mut(&mut curve_data)?;

        verify_key(owner.key, &curve.owner, "owner")?;

        Curve::check_params(&params)?;
        curve.set_params(params);

        Ok(())
    }

    #[inline(never)]
    fn delete_curve(&self) -> Result<(), CurvyError> {
        msg!("delete_curve ix");
        let DeleteCurveAccounts { curve, owner } =
            DeleteCurveAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let mut curve_data = curve.data.borrow_mut();
        let unpacked_curve = Curve::try_from_bytes_mut(&mut curve_data)?;

        verify_key(owner.key, &unpacked_curve.owner, "owner")?;

        let balance = {
            let lamports_data = curve.lamports.borrow();
            **lamports_data
        };

        transfer_lamports(curve, owner, balance)?;

        Ok(())
    }
}

/// Transfers `amount` lamports from `from_account` (must be program owned)
/// to another `to_account`. The `to_account` can be owned by anyone else.
pub fn transfer_lamports(
    from_account: &AccountInfo<'_>,
    to_account: &AccountInfo<'_>,
    amount: u64,
) -> CurvyResult<()> {
    if **from_account
        .try_borrow_lamports()
        .map_err(|_| CurvyError::OperationCanNotBePerformed)?
        < amount
    {
        return Err(CurvyError::OperationCanNotBePerformed);
    }

    **from_account
        .try_borrow_mut_lamports()
        .map_err(|_| CurvyError::OperationCanNotBePerformed)? -= amount;
    **to_account
        .try_borrow_mut_lamports()
        .map_err(|_| CurvyError::OperationCanNotBePerformed)? += amount;

    msg!(
        "transfer_lamports {} from {} to {}",
        amount,
        from_account.key,
        to_account.key
    );

    Ok(())
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &crate::ID {
        msg!("IX in not for PriceProxy but for {}", program_id);
        return Err(ProgramError::IncorrectProgramId);
    }

    Processor::new(program_id, accounts)
        .process_instruction(instruction_data)
        .map_err(|err| {
            msg!("Error: {}", err);
            err.into()
        })
}

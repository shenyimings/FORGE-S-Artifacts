use crate::error::PriceProxyError;
use crate::PriceProxyResult;
use solana_program::account_info::AccountInfo;
use solana_program::msg;

/// Transfers `amount` lamports from `from_account` (must be program owned)
/// to another `to_account`. The `to_account` can be owned by anyone else.
pub fn transfer_lamports(
    from_account: &AccountInfo<'_>,
    to_account: &AccountInfo<'_>,
    amount: u64,
) -> PriceProxyResult<()> {
    if **from_account
        .try_borrow_lamports()
        .map_err(|_| PriceProxyError::OperationCanNotBePerformed)?
        < amount
    {
        return Err(PriceProxyError::NotEnoughBalance);
    }

    **from_account
        .try_borrow_mut_lamports()
        .map_err(|_| PriceProxyError::OperationCanNotBePerformed)? -= amount;
    **to_account
        .try_borrow_mut_lamports()
        .map_err(|_| PriceProxyError::OperationCanNotBePerformed)? += amount;

    msg!(
        "transfer_lamports {} from {} to {}",
        amount,
        from_account.key,
        to_account.key
    );

    Ok(())
}

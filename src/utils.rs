use pinocchio::{account_info::AccountInfo, sysvars::{rent::Rent, Sysvar}, ProgramResult};

/// Resize an account and handle lamport transfers based on the new size
/// 
/// This function will:
/// 1. Calculate the new minimum balance required for rent exemption
/// 2. Transfer lamports if the new size requires more or less balance
/// 3. Reallocate the account to the new size
/// 
/// # Arguments
/// * `target_account` - The account to resize
/// * `authority` - The authority account that will receive excess lamports or provide additional lamports
/// * `new_size` - The new size for the account
/// * `zero_out` - Whether to zero out the new space (true if shrinking, false if expanding)
pub fn resize_account(
    target_account: &AccountInfo,
    authority: &AccountInfo,
    new_size: usize,
    zero_out: bool,
) -> ProgramResult {
    // If the account is already the correct size, return early
    if new_size == target_account.data_len() {
        return Ok(());
    }

    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);
    let current_minimum_balance = rent.minimum_balance(target_account.data_len());

    // Handle lamport transfers
    if new_minimum_balance > current_minimum_balance {
        // Need more lamports for rent exemption
        let lamports_diff = new_minimum_balance.saturating_sub(current_minimum_balance);
        *authority.try_borrow_mut_lamports()? = authority.lamports().saturating_sub(lamports_diff);
        *target_account.try_borrow_mut_lamports()? = target_account.lamports().saturating_add(lamports_diff);
    } else if new_minimum_balance < current_minimum_balance {
        // Can return excess lamports to authority
        let lamports_diff = current_minimum_balance.saturating_sub(new_minimum_balance);
        *authority.try_borrow_mut_lamports()? = authority.lamports().saturating_add(lamports_diff);
        *target_account.try_borrow_mut_lamports()? = target_account.lamports().saturating_sub(lamports_diff);
    }

    // Reallocate the account
    target_account.realloc(new_size, zero_out)?;

    Ok(())
} 
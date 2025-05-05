use pinocchio::{account_info::AccountInfo, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::Transfer;
use pinocchio::program_error::ProgramError;
use core::mem::size_of;

pub struct Context<'info> {
    pub accounts: &'info [AccountInfo],
    pub data: &'info [u8]
}

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

    // Calculate rent requirements
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);

    // First handle lamport transfers
    if new_minimum_balance > target_account.lamports() {
        // Need more lamports for rent exemption
        let lamports_diff = new_minimum_balance.saturating_sub(target_account.lamports());        
        Transfer {
            from: authority,
            to: target_account,
            lamports: lamports_diff,
        }.invoke()?;
    } else if new_minimum_balance < target_account.lamports() {
        // Can return excess lamports to authority
        let lamports_diff = target_account.lamports().saturating_sub(new_minimum_balance);
        *authority.try_borrow_mut_lamports()? = authority.lamports().saturating_add(lamports_diff);
        *target_account.try_borrow_mut_lamports()? = target_account.lamports().saturating_sub(lamports_diff);
    }

    // Now reallocate the account
    target_account.realloc(new_size, zero_out)?;

    Ok(())
}

pub struct ByteReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn new_with_minimum_size(data: &'a [u8], minimum_size: usize) -> Result<Self, ProgramError> {
        if data.len() < minimum_size {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        Ok(Self { data, offset: 0 })
    }

    pub fn read<T: Sized + Copy>(&mut self) -> Result<T, ProgramError> {
        let size = size_of::<T>();
        
        if self.offset + size > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        let value = unsafe {
            let ptr = self.data[self.offset..].as_ptr() as *const T;
            *ptr
        };
        
        self.offset += size;
        Ok(value)
    }

    pub fn read_optional<T: Sized + Copy>(&mut self) -> Result<Option<T>, ProgramError> {
        if self.offset >= self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        if self.data[self.offset] == 0 {
            self.offset += 1;
            Ok(None)
        } else {
            self.offset += 1;
            Ok(Some(self.read()?))
        }
    }

    pub fn read_str(&mut self, len: usize) -> Result<&'a str, ProgramError> {
        if self.offset + len > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let str_bytes = &self.data[self.offset..self.offset + len];
        let str = core::str::from_utf8(str_bytes)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        
        self.offset += len;
        Ok(str)
    }

    pub fn read_str_with_length(&mut self) -> Result<&'a str, ProgramError> {
        let len: u8 = self.read()?;

        self.read_str(len as usize)
    }

    pub fn remaining_bytes(&self) -> usize {
        self.data.len() - self.offset
    }
}

pub struct ByteWriter<'a> {
    data: &'a mut [u8],
    offset: usize,
}

impl<'a> ByteWriter<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn new_with_minimum_size(data: &'a mut [u8], minimum_size: usize) -> Result<Self, ProgramError> {
        if data.len() < minimum_size {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { data, offset: 0 })
    }

    pub fn write<T: Sized + Copy>(&mut self, value: T) -> Result<(), ProgramError> {
        let size = size_of::<T>();
        if self.offset + size > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        unsafe {
            let ptr = self.data[self.offset..].as_mut_ptr() as *mut T;
            *ptr = value;
        }

        self.offset += size;
        Ok(())
    }

    pub fn write_optional<T: Sized + Copy>(&mut self, value: Option<T>) -> Result<(), ProgramError> {
        let size = size_of::<T>();
        if self.offset + size + 1 > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        match value {
            Some(v) => {
                self.data[self.offset] = 1;
                self.offset += 1;
                self.write(v)
            }
            None => {
                self.data[self.offset] = 0;
                self.offset += 1;
                // Fill the remaining space with zeros
                self.data[self.offset..self.offset + size].fill(0);
                self.offset += size;
                Ok(())
            }
        }
    }

    pub fn write_str(&mut self, str: &str) -> Result<(), ProgramError> {
        if self.offset + str.len() > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        self.data[self.offset..self.offset + str.len()].copy_from_slice(str.as_bytes());
        self.offset += str.len();
        Ok(())
    }

    pub fn write_str_with_length(&mut self, str: &str) -> Result<(), ProgramError> {
        let len: u8 = str.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;
        self.write(len)?;
        self.write_str(str)
    }

    pub fn remaining_bytes(&self) -> usize {
        self.data.len() - self.offset
    }
} 
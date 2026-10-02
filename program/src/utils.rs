use core::mem::size_of;
use pinocchio::{
    account::RefMut,
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult, Resize,
};
use pinocchio_system::instructions::Transfer;
pub struct Context<'info> {
    pub accounts: &'info [AccountView],
    pub data: &'info [u8],
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
/// * `payer` - The account that will receive excess lamports or provide additional lamports
/// * `new_size` - The new size for the account
pub fn resize_account(target_account: &mut AccountView, payer: &mut AccountView, new_size: usize) -> ProgramResult {
    // Check if the new size is bigger than 10KB
    if new_size > 1024 * 10 {
        return Err(ProgramError::InvalidAccountData);
    }

    // If the account is already the correct size, return early
    if new_size == target_account.data_len() {
        return Ok(());
    }

    // Calculate rent requirements
    let rent = Rent::get()?;
    let new_minimum_balance = rent.try_minimum_balance(new_size)?;

    // First handle lamport transfers
    match new_minimum_balance.cmp(&target_account.lamports()) {
        core::cmp::Ordering::Greater => {
            // Need more lamports for rent exemption
            let lamports_diff = new_minimum_balance.saturating_sub(target_account.lamports());
            Transfer { from: payer, to: target_account, lamports: lamports_diff }.invoke()?;
        }
        core::cmp::Ordering::Less => {
            // Can return excess lamports to payer
            let lamports_diff = target_account.lamports().saturating_sub(new_minimum_balance);
            payer.set_lamports(payer.lamports().saturating_add(lamports_diff));
            target_account.set_lamports(target_account.lamports().saturating_sub(lamports_diff));
        }
        core::cmp::Ordering::Equal => {
            // No lamport transfer needed
        }
    }

    // Now reallocate the account
    target_account.resize(new_size)?;

    Ok(())
}

pub struct ByteReader<'info> {
    data: &'info [u8],
    offset: usize,
}

impl<'info> ByteReader<'info> {
    pub fn new(data: &'info [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn new_with_offset(data: &'info [u8], offset: usize) -> Self {
        Self { data, offset }
    }

    pub fn read<T: Sized + Copy>(&mut self) -> Result<T, ProgramError> {
        let size = size_of::<T>();

        if self.offset + size > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let value = unsafe { (self.data[self.offset..].as_ptr() as *const T).read_unaligned() };

        self.offset += size;
        Ok(value)
    }

    pub fn read_str(&mut self, len: usize) -> Result<&'info str, ProgramError> {
        let str_bytes = self.read_bytes(len)?;
        let str = core::str::from_utf8(str_bytes).map_err(|_| ProgramError::InvalidInstructionData)?;
        Ok(str)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'info [u8], ProgramError> {
        if self.offset + len > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let bytes = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Ok(bytes)
    }

    pub fn read_str_with_length(&mut self) -> Result<&'info str, ProgramError> {
        let len: u8 = self.read()?;

        self.read_str(len as usize)
    }

    pub fn read_bytes_with_length(&mut self) -> Result<&'info [u8], ProgramError> {
        let len: u8 = self.read()?;

        self.read_bytes(len as usize)
    }

    pub fn read_with_offset<T: Sized + Copy>(data: &'info [u8], offset: usize) -> Result<T, ProgramError> {
        let size = size_of::<T>();

        if offset + size > data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let value = unsafe { (data[offset..].as_ptr() as *const T).read_unaligned() };

        Ok(value)
    }

    pub fn read_bool_with_offset(data: &'info [u8], offset: usize) -> Result<bool, ProgramError> {
        match Self::read_with_offset::<u8>(data, offset)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

    pub fn remaining_bytes(&self) -> usize {
        self.data.len() - self.offset
    }
}

pub struct ByteWriter<'info> {
    data: &'info mut [u8],
    offset: usize,
}

impl<'info> ByteWriter<'info> {
    pub fn new_with_offset(data: &'info mut [u8], offset: usize) -> Self {
        Self { data, offset }
    }

    pub fn write<T: Sized + Copy>(&mut self, value: T) -> Result<(), ProgramError> {
        let size = size_of::<T>();
        if self.offset + size > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        unsafe { (self.data[self.offset..].as_mut_ptr() as *mut T).write_unaligned(value) };

        self.offset += size;
        Ok(())
    }

    pub fn write_str(&mut self, str: &str) -> Result<(), ProgramError> {
        self.write_bytes(str.as_bytes())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProgramError> {
        if self.offset + bytes.len() > self.data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        self.data[self.offset..self.offset + bytes.len()].copy_from_slice(bytes);
        self.offset += bytes.len();
        Ok(())
    }

    pub fn write_str_with_length(&mut self, str: &str) -> Result<(), ProgramError> {
        self.write_bytes_with_length(str.as_bytes())
    }

    pub fn write_bytes_with_length(&mut self, bytes: &[u8]) -> Result<(), ProgramError> {
        let len: u8 = bytes.len().try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;
        self.write(len)?;
        self.write_bytes(bytes)
    }

    pub fn write_with_offset<T: Sized + Copy>(
        data: &mut RefMut<'_, [u8]>,
        offset: usize,
        value: T,
    ) -> Result<(), ProgramError> {
        if offset + size_of::<T>() > data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        unsafe { (data[offset..].as_mut_ptr() as *mut T).write_unaligned(value) };

        Ok(())
    }

    pub fn remaining_bytes(&self) -> usize {
        self.data.len() - self.offset
    }
}

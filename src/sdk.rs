use pinocchio::account_info::AccountInfo;

pub struct Context<'info> {
    pub accounts: &'info [AccountInfo],
    pub data: &'info [u8]
}
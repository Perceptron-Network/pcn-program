use anchor_lang::prelude::*;
use core::mem::size_of;

#[account]
#[derive(Debug)]
pub struct SolReserve {}

impl SolReserve {
    pub const LEN: usize = 8 + size_of::<Self>();
}

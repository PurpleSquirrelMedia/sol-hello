use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::{convert::TryInto, mem::size_of};

use crate::error::GreetingError::InvalidInstruction;

#[derive(Debug)]
pub enum GreetingInstruction {
    /// Stores the number of greetings to increment the counter by
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[writeable]` The account that has the greeting counter data in it
    InitGreeting {
        /// The count of greetings to increment counter by
        num_greetings: u32,
    },
}
impl GreetingInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitGreeting {
                num_greetings: Self::unpack_greeting(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_greeting(input: &[u8]) -> Result<u32, ProgramError> {
        let num_greetings = input
            .get(..4)
            .and_then(|slice| slice.try_into().ok())
            .map(u32::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(num_greetings)
    }
    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match *self {
            Self::InitGreeting { num_greetings } => {
                buf.push(0);
                buf.extend_from_slice(&num_greetings.to_le_bytes());
            }
        }
        buf
    }
}

/// Creates an 'InitGreeting' instruction.
pub fn init_greeting(
    program_id: Pubkey,
    greeting_account_id: Pubkey,
    num_greetings: u32,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(greeting_account_id, false)],
        data: GreetingInstruction::InitGreeting { num_greetings }.pack(),
    }
}

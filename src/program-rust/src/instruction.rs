use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    pubkey::Pubkey,
};

use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct InitGreetingArgs {
    pub num_greetings: u32,
    pub greeting_string: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum GreetingInstruction {
    /// Stores the number of greetings to increment the counter by
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[writeable]` The account that has the greeting counter data in it
    InitGreeting(InitGreetingArgs),

    // Including this here to show how serialization / deserialization works with
    // borsh try_to_vec and try_from_slice where it adds the enum variant
    InitGreeting2(InitGreetingArgs),
}

// Leaving this here for reference as an alternative method for matching
// instructions. This method matches based on the first byte of the data
// and then parses the remaining bytes based on explicit references.
//
// impl GreetingInstruction {
//     pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
//         let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

//         Ok(match tag {
//             0 => Self::InitGreeting {
//                 num_greetings: Self::unpack_greeting(rest)?,
//             },
//             _ => return Err(InvalidInstruction.into()),
//         })
//     }

//     fn unpack_greeting(input: &[u8]) -> Result<u32, ProgramError> {
//         let num_greetings = input
//             .get(..4)
//             .and_then(|slice| slice.try_into().ok())
//             .map(u32::from_le_bytes)
//             .ok_or(InvalidInstruction)?;
//         Ok(num_greetings)
//     }
//     fn pack(&self) -> Vec<u8> {
//         let mut buf = Vec::with_capacity(size_of::<Self>());
//         match *self {
//             Self::InitGreeting { num_greetings } => {
//                 buf.push(0);
//                 buf.extend_from_slice(&num_greetings.to_le_bytes());
//             }
//         }
//         buf
//     }
// }

/// Creates an 'InitGreeting' instruction.
pub fn init_greeting(
    program_id: Pubkey,
    greeting_account_id: Pubkey,
    num_greetings: u32,
    greeting_string: String,
) -> Instruction {
    let greeting_instruction = GreetingInstruction::InitGreeting(InitGreetingArgs {
        num_greetings,
        greeting_string,
    });

    msg!(
        "{:?}, {:?}",
        greeting_instruction,
        greeting_instruction.try_to_vec().unwrap()
    );
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(greeting_account_id, false)],
        data: greeting_instruction.try_to_vec().unwrap(),
    }
}

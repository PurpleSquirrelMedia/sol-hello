use crate::error::GreetingError::InvalidInstruction;
use crate::instruction::GreetingInstruction;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};

/// Define the type of state stored in accounts
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GreetingAccount {
    /// number of greetings
    pub counter: u32,
    pub counter_times_2: u32,
}

impl Sealed for GreetingAccount {}
impl IsInitialized for GreetingAccount {
    fn is_initialized(&self) -> bool {
        self.counter != 0
    }
}

// Leaving this here for reference as an alternative method for matching
// instructions. This method matches based on the first byte of the data
// and then parses the remaining bytes based on explicit references.
//
// const GREETING_ACCOUNT_LEN: usize = 8; // 4 + 4
// impl Pack for GreetingAccount {
//     const LEN: usize = GREETING_ACCOUNT_LEN;

//     fn pack_into_slice(&self, output: &mut [u8]) {
//         let output = array_mut_ref![output, 0, GREETING_ACCOUNT_LEN];
//         let (counter, counter_times_2) = mut_array_refs![output, 4, 4];

//         *counter = self.counter.to_le_bytes();
//         *counter_times_2 = self.counter_times_2.to_le_bytes();
//     }

//     /// Unpacks a byte buffer into a GreetingAccount
//     fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
//         let input = array_ref![input, 0, GREETING_ACCOUNT_LEN];
//         let (counter, counter_times_2) = array_refs![input, 4, 4];

//         Ok(Self {
//             counter: u32::from_le_bytes(*counter),
//             counter_times_2: u32::from_le_bytes(*counter_times_2),
//         })
//     }
// }

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = GreetingInstruction::try_from_slice(instruction_data)
            .map_err(|_| InvalidInstruction)?;

        match instruction {
            GreetingInstruction::InitGreeting(args) => {
                msg!("Instruction: InitGreeting");
                Self::process_greeting(
                    accounts,
                    args.num_greetings,
                    args.greeting_string,
                    program_id,
                )
            }
            GreetingInstruction::InitGreeting2(_) => Err(ProgramError::Custom(42 as u32)),
        }
    }
    fn process_greeting(
        accounts: &[AccountInfo],
        num_greetings: u32,
        greeting_string: String,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let account = next_account_info(account_info_iter)?;
        msg!(
            "Unpacked {} greetings and {} string!",
            num_greetings,
            greeting_string
        );
        if account.owner != program_id {
            msg!("Greeted account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }
        // Increment and store the number of times the account has been greeted
        let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
        greeting_account.counter += &num_greetings;
        greeting_account.counter_times_2 = greeting_account.counter * 2;
        greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;
        msg!("Greeted {} time(s)!", greeting_account.counter);
        msg!(
            "Greetings times 2 equals {}!",
            greeting_account.counter_times_2
        );
        Ok(())
    }
}

// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::instruction::InitGreetingArgs;
    use solana_program::clock::Epoch;
    use std::mem;

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; mem::size_of::<u32>() * 2];
        let owner = Pubkey::default();
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );

        let instruction_data = GreetingInstruction::InitGreeting(InitGreetingArgs {
            num_greetings: 1,
            greeting_string: String::from("hello"),
        })
        .try_to_vec()
        .unwrap();

        // let mut instruction_data: Vec<u8> = 1.try_to_vec().unwrap();
        // let mut greeting_string: Vec<u8> = String::from("hello").try_to_vec().unwrap();
        // instruction_data.append(&mut greeting_string);

        let accounts = vec![account];

        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            0
        );
        Processor::process(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            1
        );
        Processor::process(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            2
        );
    }
}

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::instruction::GreetingInstruction;

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
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

const GREETING_ACCOUNT_LEN: usize = 8; // 4 + 4
impl Pack for GreetingAccount {
    const LEN: usize = GREETING_ACCOUNT_LEN;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, GREETING_ACCOUNT_LEN];
        let (counter, counter_times_2) = mut_array_refs![output, 4, 4];

        *counter = self.counter.to_le_bytes();
        *counter_times_2 = self.counter_times_2.to_le_bytes();
    }

    /// Unpacks a byte buffer into a GreetingAccount
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, GREETING_ACCOUNT_LEN];
        let (counter, counter_times_2) = array_refs![input, 4, 4];

        Ok(Self {
            counter: u32::from_le_bytes(*counter),
            counter_times_2: u32::from_le_bytes(*counter_times_2),
        })
    }
}

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = GreetingInstruction::unpack(instruction_data)?;

        match instruction {
            GreetingInstruction::InitGreeting { num_greetings } => {
                msg!("Instruction: InitGreeting");
                Self::process_greeting(accounts, num_greetings, program_id)
            }
        }
    }
    fn process_greeting(
        accounts: &[AccountInfo],
        num_greetings: u32,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let account = next_account_info(account_info_iter)?;
        msg!("Unpacked {} greetings!", num_greetings);
        if account.owner != program_id {
            msg!("Greeted account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }
        // Increment and store the number of times the account has been greeted
        let mut greeting_account = GreetingAccount::unpack_unchecked(&account.data.borrow())?;
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
        let mut instruction_data: Vec<u8> = vec![0, 1];
        let mut num_greetings: Vec<u8> = vec![0; 31];
        instruction_data.append(&mut num_greetings);

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

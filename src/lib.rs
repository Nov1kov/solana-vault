use solana_program::{
    account_info::{ AccountInfo, next_account_info },
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    system_instruction,
    system_program,
    program_error::ProgramError,
};
use borsh::{ BorshSerialize, BorshDeserialize };

// Define the structure for storing deposit information
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default)]
pub struct DepositAccount {
    pub owner: Pubkey,
    pub balance: u64,
}

// Program instructions
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum DepositInstruction {
    /// Deposit SOL into the account
    Deposit {
        amount: u64,
    },
    /// Withdraw SOL from the account
    Withdraw {
        amount: u64,
    },
}

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;

// Define the program's entrypoint
entrypoint!(process_instruction);


// Main program logic
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Deserialize the instruction
    let instruction = DepositInstruction::try_from_slice(instruction_data)?;

    // Validate accounts
    let accounts_iter = &mut accounts.iter();
    let depositor = next_account_info(accounts_iter)?;
    let deposit_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Ensure the depositor is the signer
    if !depositor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the correct system program
    if *system_program.key != system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Load or initialize the deposit account
    let mut deposit_data = DepositAccount::try_from_slice(&deposit_account.data.borrow())?;

    // Проверка, что depositor является владельцем аккаунта
    if deposit_data.owner == Pubkey::default() {
        // Первая инициализация - устанавливаем владельца
        deposit_data.owner = *depositor.key;
    } else if deposit_data.owner != *depositor.key {
        // Если попытка использовать чужой аккаунт
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Process the instruction
    match instruction {
        DepositInstruction::Deposit { amount } => {
            // Transfer SOL from depositor to deposit account
            if amount == 0 {
                return Err(ProgramError::InvalidInstructionData);
            }

            // Invoke system program to transfer funds
            solana_program::program::invoke(
                &system_instruction::transfer(depositor.key, deposit_account.key, amount),
                &[depositor.clone(), deposit_account.clone(), system_program.clone()]
            )?;

            // Update balance
            deposit_data.balance += amount;
            deposit_data.serialize(&mut &mut deposit_account.data.borrow_mut()[..])?;

            msg!("Deposit successful: {} SOL", amount);
        },
        DepositInstruction::Withdraw { amount } => {
            // Check sufficient balance
            if amount > deposit_data.balance {
                return Err(ProgramError::InsufficientFunds);
            }

            // Transfer SOL back to depositor
            **deposit_account.try_borrow_mut_lamports()? -= amount;
            **depositor.try_borrow_mut_lamports()? += amount;

            // Update balance
            deposit_data.balance -= amount;
            deposit_data.serialize(&mut &mut deposit_account.data.borrow_mut()[..])?;

            msg!("Withdrawal successful: {} SOL", amount);
        }
    }

    Ok(())
}


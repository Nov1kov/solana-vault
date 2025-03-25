use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint, entrypoint::ProgramResult, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::Pubkey, system_instruction, system_program, sysvar::{rent::Rent, Sysvar}};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default)]
pub struct DepositAccount {
    pub owner: Pubkey,
    pub balance: u64,
}

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

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = DepositInstruction::try_from_slice(instruction_data)?;
    let accounts_iter = &mut accounts.iter();
    let depositor = next_account_info(accounts_iter)?;
    let deposit_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !depositor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if *system_program.key != system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (deposit_account_key, bump) = Pubkey::find_program_address(&[b"deposit", depositor.key.as_ref()], program_id);

    if *deposit_account.key != deposit_account_key {
        return Err(ProgramError::InvalidAccountData);
    }

    if deposit_account.lamports() == 0 {
        let rent = Rent::get()?.minimum_balance(std::mem::size_of::<DepositAccount>());
        invoke_signed(
            &system_instruction::create_account(
                depositor.key,
                &deposit_account_key,
                rent,
                std::mem::size_of::<DepositAccount>() as u64,
                program_id,
            ),
            &[depositor.clone(), deposit_account.clone(), system_program.clone()],
            &[&[b"deposit", depositor.key.as_ref(), &[bump]]],
        )?;
    }

    let mut deposit_data = DepositAccount::try_from_slice(&deposit_account.data.borrow())?;
    if deposit_data.owner == Pubkey::default() {
        deposit_data.owner = *depositor.key;
    } else if deposit_data.owner != *depositor.key {
        return Err(ProgramError::MissingRequiredSignature);
    }

    match instruction {
        DepositInstruction::Deposit { amount } => {
            if amount == 0 {
                return Err(ProgramError::InvalidInstructionData);
            }
            invoke(
                &system_instruction::transfer(depositor.key, &deposit_account_key, amount),
                &[depositor.clone(), deposit_account.clone(), system_program.clone()],
            )?;
            deposit_data.balance += amount;
        }
        DepositInstruction::Withdraw { amount } => {
            if amount > deposit_data.balance {
                return Err(ProgramError::InsufficientFunds);
            }
            **deposit_account.try_borrow_mut_lamports()? -= amount;
            **depositor.try_borrow_mut_lamports()? += amount;
            deposit_data.balance -= amount;
        }
    }
    deposit_data.serialize(&mut &mut deposit_account.data.borrow_mut()[..])?;
    Ok(())
}

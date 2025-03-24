use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    system_instruction,
    system_program,
    program_error::ProgramError,
};
use borsh::{BorshSerialize, BorshDeserialize};

// Define the structure for storing deposit information
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default)]
pub struct DepositAccount {
    pub balance: u64,
}

// Program instructions
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum DepositInstruction {
    /// Deposit SOL into the account
    Deposit { amount: u64 },
    /// Withdraw SOL from the account
    Withdraw { amount: u64 },
}

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

#[cfg(test)]
mod tests {
    use solana_program_test::{processor, BanksClient, ProgramTest};
    use std::str::FromStr;
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_sdk::{account::Account, signature::{Keypair, Signer}, transaction::Transaction, pubkey::Pubkey as SdkPubkey, system_program};
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use crate::{process_instruction, DepositAccount, DepositInstruction};

    #[tokio::test]
    async fn test_deposit() {
        // Create a unique program ID
        let program_id = SdkPubkey::from_str("EbKQVLUFJp38qanC4NwQUqsrWrRV4MUMhFRmTTJKHNMC").unwrap();

        // Setup the program test environment
        let mut program_test = ProgramTest::new(
            "solana_deposit_contract",
            program_id,
            processor!(process_instruction)
        );

        // Create test accounts
        let depositor = Keypair::new();
        let deposit_account = Keypair::new();

        // Add accounts to the test environment
        program_test.add_account(
            depositor.pubkey(),
            Account {
                lamports: 100_000_000, // 0.1 SOL
                ..Account::default()
            }
        );

        // Initialize the deposit account
        program_test.add_account(
            deposit_account.pubkey(),
            Account {
                owner: program_id,
                ..Account::default()
            }
        );

        // Start the test runtime
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Prepare deposit instruction
        let deposit_amount = 50_000_000; // 0.05 SOL
        let deposit_instruction = Instruction::new_with_borsh(
            program_id,
            &DepositInstruction::Deposit { amount: deposit_amount },
            vec![
                AccountMeta::new(depositor.pubkey(), true),
                AccountMeta::new(deposit_account.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ]
        );

        // Create and send transaction
        let mut transaction = Transaction::new_with_payer(
            &[deposit_instruction],
            Some(&payer.pubkey())
        );
        transaction.sign(&[&payer, &depositor], recent_blockhash);

        // Process the transaction
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify deposit account balance
        let deposit_account_data = banks_client
            .get_account(deposit_account.pubkey())
            .await
            .unwrap()
            .unwrap();

        let account_data = DepositAccount::try_from_slice(&deposit_account_data.data).unwrap();
        assert_eq!(account_data.balance, deposit_amount);
    }

    #[tokio::test]
    async fn test_withdrawal() {
        // Create a unique program ID
        let program_id = SdkPubkey::from_str("EbKQVLUFJp38qanC4NwQUqsrWrRV4MUMhFRmTTJKHNMC").unwrap();

        // Setup the program test environment
        let mut program_test = ProgramTest::new(
            "solana_deposit_contract",
            program_id,
            processor!(process_instruction)
        );

        let depositor = Keypair::new();
        let deposit_account = Keypair::new();

        // Pre-fund accounts
        program_test.add_account(
            depositor.pubkey(),
            Account {
                lamports: 100_000_000,
                ..Account::default()
            }
        );

        // Initialize deposit account with balance
        let initial_balance = 75_000_000; // 0.075 SOL
        let mut deposit_account_data = DepositAccount { balance: initial_balance };
        let mut account_data = vec![];
        deposit_account_data.serialize(&mut account_data).unwrap();

        program_test.add_account(
            deposit_account.pubkey(),
            Account {
                owner: program_id,
                lamports: initial_balance,
                data: account_data,
                ..Account::default()
            }
        );

        // Start test runtime
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Prepare withdrawal instruction
        let withdraw_amount = 50_000_000; // 0.05 SOL
        let deposit_pubkey = depositor.pubkey();
        let withdraw_instruction = Instruction::new_with_borsh(
            program_id,
            &DepositInstruction::Withdraw { amount: withdraw_amount },
            vec![
                AccountMeta::new(deposit_pubkey, true),
                AccountMeta::new(deposit_account.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ]
        );

        // Create and send transaction
        let mut transaction = Transaction::new_with_payer(
            &[withdraw_instruction],
            Some(&payer.pubkey())
        );
        transaction.sign(&[&payer, &depositor], recent_blockhash);

        // Process the transaction
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify deposit account balance after withdrawal
        let deposit_account_data = banks_client
            .get_account(deposit_account.pubkey())
            .await
            .unwrap()
            .unwrap();

        let account_data = DepositAccount::try_from_slice(&deposit_account_data.data).unwrap();
        assert_eq!(account_data.balance, initial_balance - withdraw_amount);
    }

    #[tokio::test]
    async fn test_insufficient_funds_withdrawal() {
        // Create a unique program ID
        let program_id = SdkPubkey::from_str("EbKQVLUFJp38qanC4NwQUqsrWrRV4MUMhFRmTTJKHNMC").unwrap();

        // Setup the program test environment
        let mut program_test = ProgramTest::new(
            "solana_deposit_contract",
            program_id,
            processor!(process_instruction)
        );

        let depositor = Keypair::new();
        let deposit_account = Keypair::new();

        // Pre-fund accounts
        program_test.add_account(
            depositor.pubkey(), 
            Account { 
                lamports: 100_000_000, 
                ..Account::default() 
            }
        );

        // Initialize deposit account with low balance
        let initial_balance = 25_000_000; // 0.025 SOL
        let mut deposit_account_data = DepositAccount { balance: initial_balance };
        let mut account_data = vec![];
        deposit_account_data.serialize(&mut account_data).unwrap();

        program_test.add_account(
            deposit_account.pubkey(), 
            Account { 
                owner: program_id,
                lamports: initial_balance,
                data: account_data,
                ..Account::default() 
            }
        );

        // Start test runtime
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Prepare withdrawal instruction with amount exceeding balance
        let withdraw_amount = 50_000_000; // 0.05 SOL
        let withdraw_instruction = Instruction::new_with_borsh(
            program_id,
            &DepositInstruction::Withdraw { amount: withdraw_amount },
            vec![
                AccountMeta::new(depositor.pubkey(), true),
                AccountMeta::new(deposit_account.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ]
        );

        // Create and send transaction
        let mut transaction = Transaction::new_with_payer(
            &[withdraw_instruction],
            Some(&payer.pubkey())
        );
        transaction.sign(&[&payer, &depositor], recent_blockhash);

        // Expect transaction to fail due to insufficient funds
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_err());
    }
}

// Required by Solana for proper program compilation
#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
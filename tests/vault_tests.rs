#[cfg(test)]
mod tests {
    use solana_program_test::{processor, BanksClientError, ProgramTest};
    use std::str::FromStr;
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::pubkey::Pubkey;
    use solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction, InstructionError},
        pubkey::Pubkey as SdkPubkey,
        signature::{Keypair, Signer},
        system_program,
        transaction::{Transaction, TransactionError},
    };
    use solana_vault::{process_instruction, DepositAccount, DepositInstruction};

    fn setup() -> (Pubkey, ProgramTest, Keypair) {
        let program_id = SdkPubkey::from_str(
            "EbKQVLUFJp38qanC4NwQUqsrWrRV4MUMhFRmTTJKHNMC"
        ).unwrap();

        let mut program_test = ProgramTest::new(
            "solana_deposit_contract",
            program_id,
            processor!(process_instruction),
        );

        let depositor = Keypair::new();

        program_test.add_account(depositor.pubkey(), Account {
            lamports: 100_000_000,
            ..Account::default()
        });
        (program_id, program_test, depositor)
    }

    #[tokio::test]
    async fn test_deposit() {
        let (program_id, mut program_test, depositor) = setup();

        let deposit_account = Keypair::new();

        let deposit_account_data = DepositAccount { owner: depositor.pubkey(), balance: 0 };
        let mut data = vec![];
        deposit_account_data.serialize(&mut data).unwrap();
        program_test.add_account(deposit_account.pubkey(), Account {
            owner: program_id,
            lamports: 1,
            data,
            ..Account::default()
        });

        let (banks_client, payer, recent_blockhash) = program_test.start().await;

        let deposit_amount = 50_000_000; // 0.05 SOL
        let deposit_instruction = Instruction::new_with_borsh(
            program_id,
            &(DepositInstruction::Deposit { amount: deposit_amount }),
            vec![
                AccountMeta::new(depositor.pubkey(), true),
                AccountMeta::new(deposit_account.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false)
            ],
        );

        let mut transaction = Transaction::new_with_payer(
            &[deposit_instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &depositor], recent_blockhash);

        banks_client.process_transaction(transaction).await.unwrap();

        // Verify deposit account balance
        let deposit_account_data = banks_client
            .get_account(deposit_account.pubkey()).await
            .unwrap()
            .unwrap();

        let account_data = DepositAccount::try_from_slice(&deposit_account_data.data).unwrap();
        assert_eq!(account_data.balance, deposit_amount);
    }

    #[tokio::test]
    async fn test_withdrawal() {
        let (program_id, mut program_test, depositor) = setup();

        let deposit_account = Keypair::new();

        let initial_balance = 75_000_000; // 0.075 SOL
        let deposit_account_data = DepositAccount { owner: depositor.pubkey(), balance: initial_balance };
        let mut account_data = vec![];
        deposit_account_data.serialize(&mut account_data).unwrap();

        program_test.add_account(deposit_account.pubkey(), Account {
            owner: program_id,
            lamports: initial_balance,
            data: account_data,
            ..Account::default()
        });

        let (banks_client, payer, recent_blockhash) = program_test.start().await;

        let withdraw_amount = 50_000_000; // 0.05 SOL
        let deposit_pubkey = depositor.pubkey();
        let withdraw_instruction = Instruction::new_with_borsh(
            program_id,
            &(DepositInstruction::Withdraw { amount: withdraw_amount }),
            vec![
                AccountMeta::new(deposit_pubkey, true),
                AccountMeta::new(deposit_account.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false)
            ],
        );

        let mut transaction = Transaction::new_with_payer(
            &[withdraw_instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &depositor], recent_blockhash);

        banks_client.process_transaction(transaction).await.unwrap();

        // Verify deposit account balance after withdrawal
        let deposit_account_data = banks_client
            .get_account(deposit_account.pubkey()).await
            .unwrap()
            .unwrap();

        let account_data = DepositAccount::try_from_slice(&deposit_account_data.data).unwrap();
        assert_eq!(account_data.balance, initial_balance - withdraw_amount);
    }

    #[tokio::test]
    async fn test_insufficient_funds_withdrawal() {
        let (program_id, mut program_test, depositor) = setup();

        let deposit_account = Keypair::new();

        let initial_balance = 25_000_000; // 0.025 SOL
        let deposit_account_data = DepositAccount { owner: depositor.pubkey(), balance: initial_balance };
        let mut account_data = vec![];
        deposit_account_data.serialize(&mut account_data).unwrap();

        program_test.add_account(deposit_account.pubkey(), Account {
            owner: program_id,
            lamports: initial_balance,
            data: account_data,
            ..Account::default()
        });

        let (banks_client, payer, recent_blockhash) = program_test.start().await;

        // Prepare withdrawal instruction with amount exceeding balance
        let withdraw_amount = 50_000_000; // 0.05 SOL
        let withdraw_instruction = Instruction::new_with_borsh(
            program_id,
            &(DepositInstruction::Withdraw { amount: withdraw_amount }),
            vec![
                AccountMeta::new(depositor.pubkey(), true),
                AccountMeta::new(deposit_account.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false)
            ],
        );

        let mut transaction = Transaction::new_with_payer(
            &[withdraw_instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &depositor], recent_blockhash);

        let result = banks_client.process_transaction(transaction).await;
        if
        let Err(
            BanksClientError::TransactionError(
                TransactionError::InstructionError(0, InstructionError::InsufficientFunds),
            ),
        ) = result
        {
            // Test passed
        } else {
            panic!("Expected InsufficientFunds error");
        }
    }

    #[tokio::test]
    async fn test_unauthorized_access() {
        let (program_id, mut program_test, depositor) = setup();

        let unauthorized_user = Keypair::new();
        let deposit_account = Keypair::new();

        program_test.add_account(unauthorized_user.pubkey(), Account {
            lamports: 100_000_000,
            ..Account::default()
        });

        let initial_balance = 75_000_000; // 0.075 SOL
        let deposit_account_data = DepositAccount {
            owner: depositor.pubkey(),
            balance: initial_balance,
        };
        let mut account_data = vec![];
        deposit_account_data.serialize(&mut account_data).unwrap();

        program_test.add_account(deposit_account.pubkey(), Account {
            owner: program_id,
            lamports: initial_balance,
            data: account_data,
            ..Account::default()
        });

        let (banks_client, payer, recent_blockhash) = program_test.start().await;

        let withdraw_amount = 50_000_000; // 0.05 SOL
        let withdraw_instruction = Instruction::new_with_borsh(
            program_id,
            &(DepositInstruction::Withdraw { amount: withdraw_amount }),
            vec![
                AccountMeta::new(unauthorized_user.pubkey(), true),
                AccountMeta::new(deposit_account.pubkey(), false),
                AccountMeta::new_readonly(system_program::id(), false)
            ],
        );

        let mut transaction = Transaction::new_with_payer(
            &[withdraw_instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &unauthorized_user], recent_blockhash);

        let result = banks_client.process_transaction(transaction).await;

        if let Err(BanksClientError::TransactionError(
                       TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
                   )) = result {
            // Тест пройден успешно
        } else {
            panic!("Expected MissingRequiredSignature error");
        }
    }
}
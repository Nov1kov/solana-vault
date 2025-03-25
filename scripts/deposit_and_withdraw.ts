import * as web3 from "@solana/web3.js";
import * as borsh from "borsh";

// Изменим структуру для точного соответствия Rust-enum
class DepositInstruction {
  kind: number;
  amount: bigint;

  constructor(type: "deposit" | "withdraw", amount: number) {
    // 0 для deposit, 1 для withdraw
    this.kind = type === "deposit" ? 0 : 1;
    this.amount = BigInt(amount);
  }

  // Обновленная схема сериализации
  static schema = new Map([
    [
      DepositInstruction,
      {
        kind: "struct",
        fields: [
          ["kind", "u8"],   // Используем u8 вместо string
          ["amount", "u64"],
        ],
      },
    ],
  ]);

  toBuffer(): Buffer {
    return Buffer.from(
      borsh.serialize(
        DepositInstruction.schema,
        // Прямая сериализация объекта
        this
      )
    );
  }
}

async function interactWithDepositProgram() {
  // Подключение к сети (в данном примере используется devnet)
  const connection = new web3.Connection(
    web3.clusterApiUrl("devnet"),
    "confirmed"
  );

  // Преобразуем Uint8Array в Buffer для совместимости
  const privateKey = Buffer.from([
    /* private key */
  ]);

  // Создание кошелька для взаимодействия
  const wallet = web3.Keypair.fromSecretKey(privateKey);

  // Адрес программы (Programme ID) - нужно заменить на реальный
  const PROGRAM_ID = pg.PROGRAM_ID;

  // Создание аккаунта для хранения депозита
  const depositAccount = web3.Keypair.generate();

  // Депозит 0.02 SOL
  const depositIx = new DepositInstruction(
    "deposit",
    web3.LAMPORTS_PER_SOL * 0.02
  );

  const depositInstruction = new web3.TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: depositAccount.publicKey, isSigner: false, isWritable: true },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: PROGRAM_ID,
    data: depositIx.toBuffer(),
  });

  console.log("делаем депозит");

  // Размер структуры DepositAccount в байтах (pubkey 32 байта + u64 8 байт)
  const ACCOUNT_SIZE = 40;

  // Создание транзакции выделения места под аккаунт
  const createAccountIx = web3.SystemProgram.createAccount({
    fromPubkey: wallet.publicKey,
    newAccountPubkey: depositAccount.publicKey,
    lamports: await connection.getMinimumBalanceForRentExemption(ACCOUNT_SIZE),
    space: ACCOUNT_SIZE,
    programId: PROGRAM_ID
  });

  // Модифицируйте транзакцию
  const tx = new web3.Transaction()
    .add(createAccountIx)  // Сначала создаем аккаунт
    .add(depositInstruction); // Затем выполняем депозит
  const txSignature = await web3.sendAndConfirmTransaction(connection, tx, [
    wallet,
    depositAccount,
  ]);

  console.log("Депозит выполнен. Транзакция:", txSignature);

  // Вывод 0.02 SOL
  const withdrawIx = new DepositInstruction(
    "withdraw",
    web3.LAMPORTS_PER_SOL * 0.02
  );
  const withdrawInstruction = new web3.TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: depositAccount.publicKey, isSigner: false, isWritable: true },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: PROGRAM_ID,
    data: withdrawIx.toBuffer(),
  });

  // Создание и отправка транзакции на вывод
  const withdrawTx = new web3.Transaction().add(withdrawInstruction);
  const withdrawTxSignature = await web3.sendAndConfirmTransaction(
    connection,
    withdrawTx,
    [wallet, depositAccount]
  );

  console.log("Вывод выполнен. Транзакция:", withdrawTxSignature);
}

// Запуск функции
interactWithDepositProgram().catch(console.error);
import * as web3 from "@solana/web3.js";
import * as borsh from "borsh";

class DepositInstruction {
  kind: number;
  amount: bigint;

  constructor(type: "deposit" | "withdraw", amount: number) {
    // 0 для deposit, 1 для withdraw
    this.kind = type === "deposit" ? 0 : 1;
    this.amount = BigInt(amount);
  }

  static schema = new Map([
    [
      DepositInstruction,
      {
        kind: "struct",
        fields: [
          ["kind", "u8"],
          ["amount", "u64"],
        ],
      },
    ],
  ]);

  toBuffer(): Buffer {
    return Buffer.from(
      borsh.serialize(
        DepositInstruction.schema,
        this
      )
    );
  }
}

async function interactWithDepositProgram() {
  const connection = new web3.Connection(
    web3.clusterApiUrl("devnet"),
    "confirmed"
  );

  const privateKey = Buffer.from([
    /* private key */
  ]);

  // Создание кошелька для взаимодействия
  const wallet = web3.Keypair.fromSecretKey(privateKey);
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

  const ACCOUNT_SIZE = 40;

  const createAccountIx = web3.SystemProgram.createAccount({
    fromPubkey: wallet.publicKey,
    newAccountPubkey: depositAccount.publicKey,
    lamports: await connection.getMinimumBalanceForRentExemption(ACCOUNT_SIZE),
    space: ACCOUNT_SIZE,
    programId: PROGRAM_ID
  });

  const tx = new web3.Transaction()
    .add(createAccountIx)
    .add(depositInstruction);
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

  const withdrawTx = new web3.Transaction().add(withdrawInstruction);
  const withdrawTxSignature = await web3.sendAndConfirmTransaction(
    connection,
    withdrawTx,
    [wallet, depositAccount]
  );

  console.log("Вывод выполнен. Транзакция:", withdrawTxSignature);
}

interactWithDepositProgram().catch(console.error);
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

  const wallet = pg.wallet.keypair;
  const PROGRAM_ID = pg.PROGRAM_ID;

  const [depositAccountPubKey, bump] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("deposit"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  // Депозит 0.02 SOL
  const depositIx = new DepositInstruction(
    "deposit",
    web3.LAMPORTS_PER_SOL * 0.02
  );

  const depositInstruction = new web3.TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
      { pubkey: depositAccountPubKey, isSigner: false, isWritable: true },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: PROGRAM_ID,
    data: depositIx.toBuffer(),
  });

  console.log("делаем депозит в", depositAccountPubKey.toString());

  const tx = new web3.Transaction()
    .add(depositInstruction);
  const txSignature = await web3.sendAndConfirmTransaction(connection, tx, [
    wallet
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
      { pubkey: depositAccountPubKey, isSigner: false, isWritable: true },
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
    [wallet]
  );

  console.log("Вывод выполнен. Транзакция:", withdrawTxSignature);
}

interactWithDepositProgram().catch(console.error);
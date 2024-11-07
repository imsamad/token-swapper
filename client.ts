import { randomBytes } from "node:crypto";

import * as anchor from "@coral-xyz/anchor";
import {
  createAccountsMintsAndTokenAccounts,
  makeKeypairs,
} from "@solana-developers/helpers";
import { TokenSwapper } from "./target/types/token_swapper";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  type TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
const provider = anchor.AnchorProvider.env();

anchor.setProvider(provider);

const payer = (provider.wallet as anchor.Wallet).payer;

const connection = provider.connection;

const program = anchor.workspace.TokenSwapper as anchor.Program<TokenSwapper>;

const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
  TOKEN_2022_PROGRAM_ID;

const tokenProgram = TOKEN_PROGRAM;

const getRandomBigNumber_ = (size = 8) => randomBytes(8);

const getRandomBigNumber = (size = 8) => {
  return new anchor.BN(randomBytes(size));
};

const getUsersMintAndTokenAccounts = () =>
  createAccountsMintsAndTokenAccounts(
    [
      [1_000_000_000, 0],
      [0, 1_000_000_000],
    ],
    1 * LAMPORTS_PER_SOL,
    connection,
    payer
  );

async function make_offer() {
  let usersMintAndTokenAccounts = await getUsersMintAndTokenAccounts();

  const maker = usersMintAndTokenAccounts.users[0];
  const taker = usersMintAndTokenAccounts.users[1];

  const mintA = usersMintAndTokenAccounts.mints[0];
  const mintB = usersMintAndTokenAccounts.mints[1];

  const makerTokenAccountA = usersMintAndTokenAccounts.tokenAccounts[0][0];
  const makerTokenAccountB = usersMintAndTokenAccounts.tokenAccounts[0][1];

  const takerTokenAccountA = usersMintAndTokenAccounts.tokenAccounts[1][0];
  const takerTokenAccountB = usersMintAndTokenAccounts.tokenAccounts[1][1];

  let offerId = getRandomBigNumber();

  const token_a_offered_amount = new anchor.BN(1_000_000);
  const token_b_demand_amount = new anchor.BN(1_000_000 / 2);

  const [offerPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("offer"),
      maker.publicKey.toBuffer(),
      offerId.toArrayLike(Buffer, "le", 8),
    ],
    program.programId
  );
  const vault = await getAssociatedTokenAddress(
    mintA.publicKey,
    offerPda,
    true,
    TOKEN_PROGRAM
  );

  console.log({
    maker: maker.publicKey.toBase58(),
    taker: taker.publicKey.toBase58(),
    mintA: mintA.publicKey.toBase58(),
    mintB: mintB.publicKey.toBase58(),
    makerTokenAccountA: makerTokenAccountA.toBase58(),
    makerTokenAccountB: makerTokenAccountB.toBase58(),
    takerTokenAccountA: takerTokenAccountA.toBase58(),
    takerTokenAccountB: takerTokenAccountB.toBase58(),
    offerPda: offerPda.toBase58(),
    vault: vault.toBase58(),
  });

  // return;
  var hash = await program.methods
    .makeOffer(offerId, token_a_offered_amount, token_b_demand_amount)
    .accounts({
      maker: maker.publicKey,
      tokenMintA: mintA.publicKey,
      tokenMintB: mintB.publicKey,
      tokenProgram,
    })
    .signers([maker])
    .rpc();

  console.log("hash: ", hash);

  hash = await program.methods
    .takeOfferLatesr()
    .accounts({
      taker: taker.publicKey,
      maker: maker.publicKey,
      tokenMintA: mintA.publicKey,
      tokenMintB: mintB.publicKey,

      takerTokenAccountB: takerTokenAccountB,
      // takerTokenAccountA: takerTokenAccountA,

      vault: vault,
      offer: offerPda,
      tokenProgram,
    })
    .signers([taker])
    .rpc();

  // console.log("take_offer: ", hash);
}

async function getAllOffers() {
  const offers = await program.account.offer.all();

  console.log(offers);
}

async function take_offer() {}

make_offer().finally(() => {
  // getAllOffers();
});

// const offerIdx = new anchor.BN(getRandomBigNumber());

// const buff = Buffer.from(
//   new Uint8Array(new anchor.BN(offerIdx).toArray("le", 8))
// );
// console.log(offerIdx);
// console.log(buff);

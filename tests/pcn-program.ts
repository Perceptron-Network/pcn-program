import {
  AnchorProvider,
  BN,
  Program,
  setProvider,
  web3,
} from "@anchor-lang/core";
import { expect } from "chai";
import { PcnProgram } from "../target/types/pcn_program";

const idl = require("../target/idl/pcn_program.json") as PcnProgram;

const TOKEN_PROGRAM_ID = new web3.PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);
const TOKEN_ACCOUNT_LEN = 165;
const TOKEN_BASE_UNITS = 1_000_000_000;
const QUALITY_PPM_SCALE = 1_000_000;
const PROGRAM_ID = new web3.PublicKey(
  "FzHRzKNFB7Mck5FHj2MXUaywAgtB2EA2EeEQEkp59Xfo"
);
const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new web3.PublicKey(
  "BPFLoaderUpgradeab1e11111111111111111111111"
);
const programData = web3.PublicKey.findProgramAddressSync(
  [PROGRAM_ID.toBuffer()],
  BPF_LOADER_UPGRADEABLE_PROGRAM_ID
)[0];

type Pcn = Program<PcnProgram>;

describe("pcn-program", () => {
  const provider = AnchorProvider.env();
  setProvider(provider);

  const program = new Program<PcnProgram>(idl, provider) as Pcn;
  const payer = provider.wallet.payer!;
  const admin = web3.Keypair.generate();
  const oracle = web3.Keypair.generate();
  const mint = web3.Keypair.generate();
  const config = pda(["config"]);
  const mintAuthority = pda(["mint_authority"]);
  const solReserve = pda(["sol_reserve"]);
  const tokenReserveVault = pda(["token_reserve"]);
  const curve = {
    maxEpochMint: new BN(TOKEN_BASE_UNITS),
    emissionMultiplierPpm: new BN(1_000_000),
    saturationUnits: new BN(100),
    historyMinted: new BN(TOKEN_BASE_UNITS),
    targetSupportLamportsPerToken: new BN(10_000_000),
    maxSupply: new BN(10 * TOKEN_BASE_UNITS),
  };

  let nextEpochId = 1;

  before(async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000),
      "confirmed"
    );

    const unauthorizedPayer = web3.Keypair.generate();
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        unauthorizedPayer.publicKey,
        2_000_000_000
      ),
      "confirmed"
    );

    await expectRejected(
      program.methods
        .initializeConfig({
          admin: admin.publicKey,
          oracle: oracle.publicKey,
          claimWindowSlots: new BN(5),
          curve,
        })
        .accountsStrict({
          payer: unauthorizedPayer.publicKey,
          program: PROGRAM_ID,
          programData,
          config,
          rewardMint: mint.publicKey,
          mintAuthority,
          solReserve,
          tokenReserveVault,
          systemProgram: web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([unauthorizedPayer, mint])
        .rpc(),
      "UnauthorizedInitializer"
    );

    await expectRejected(
      program.methods
        .initializeConfig({
          admin: admin.publicKey,
          oracle: oracle.publicKey,
          claimWindowSlots: new BN(5),
          curve,
        })
        .accountsStrict({
          payer: payer.publicKey,
          program: PROGRAM_ID,
          programData: PROGRAM_ID,
          config,
          rewardMint: mint.publicKey,
          mintAuthority,
          solReserve,
          tokenReserveVault,
          systemProgram: web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([mint])
        .rpc(),
      "AccountNotProgramData"
    );

    await program.methods
      .initializeConfig({
        admin: admin.publicKey,
        oracle: oracle.publicKey,
        claimWindowSlots: new BN(5),
        curve,
      })
      .accountsStrict({
        payer: payer.publicKey,
        program: PROGRAM_ID,
        programData,
        config,
        rewardMint: mint.publicKey,
        mintAuthority,
        solReserve,
        tokenReserveVault,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([mint])
      .rpc();
  });

  it("initializes config and enforces admin update authority", async () => {
    const loadedConfig = await program.account.config.fetch(config);
    expect(loadedConfig.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    expect(loadedConfig.oracle.toBase58()).to.equal(
      oracle.publicKey.toBase58()
    );
    expect(loadedConfig.rewardMint.toBase58()).to.equal(
      mint.publicKey.toBase58()
    );
    expect(loadedConfig.claimWindowSlots.toString()).to.equal("5");

    await expectRejected(
      program.methods
        .updateConfig({
          oracle: null,
          claimWindowSlots: new BN(5),
          curve: null,
        })
        .accountsStrict({
          admin: oracle.publicKey,
          config,
        })
        .signers([oracle])
        .rpc(),
      "UnauthorizedAdmin"
    );

    await program.methods
      .updateConfig({
        oracle: null,
        claimWindowSlots: new BN(5),
        curve: null,
      })
      .accountsStrict({
        admin: admin.publicKey,
        config,
      })
      .signers([admin])
      .rpc();

    const updated = await program.account.config.fetch(config);
    expect(updated.claimWindowSlots.toString()).to.equal("5");
  });

  it("finalizes an epoch, creates quality-weighted claims, claims, and sweeps dust", async () => {
    const fx = await createEpochFixture();
    const reserveBefore = await provider.connection.getBalance(solReserve);

    await openEpoch(fx, 10_000_000);
    await waitForSlot(provider, 2);
    await finalizeEpoch(fx, 100);

    const finalized = await program.account.epoch.fetch(fx.epoch);
    expect(finalized.rewardPoolAmount.toString()).to.equal("632120559");
    expect(finalized.totalRewardWeight.toString()).to.equal("100");
    const reserveAfter = await provider.connection.getBalance(solReserve);
    expect(reserveAfter - reserveBefore).to.equal(
      finalized.consumedSupportLamports.toNumber()
    );

    const claimOne = pda([
      "claim",
      u64(fx.epochId),
      fx.userOne.publicKey.toBuffer(),
    ]);
    const claimTwo = pda([
      "claim",
      u64(fx.epochId),
      fx.userTwo.publicKey.toBuffer(),
    ]);

    await createClaim(
      fx,
      claimOne,
      fx.userOne.publicKey,
      50,
      QUALITY_PPM_SCALE
    );
    await createClaim(fx, claimTwo, fx.userTwo.publicKey, 100, 500_000);

    const firstClaim = await program.account.claim.fetch(claimOne);
    const secondClaim = await program.account.claim.fetch(claimTwo);
    expect(firstClaim.rewardWeight.toString()).to.equal("50");
    expect(secondClaim.rewardWeight.toString()).to.equal("50");
    expect(firstClaim.rewardAmount.toString()).to.equal("316060279");
    expect(secondClaim.rewardAmount.toString()).to.equal("316060279");

    await claimReward(fx, claimOne, fx.userOne, fx.userOneToken.publicKey);
    await claimReward(fx, claimTwo, fx.userTwo, fx.userTwoToken.publicKey);

    expect(await tokenAmount(provider, fx.userOneToken.publicKey)).to.equal(
      "316060279"
    );
    expect(await tokenAmount(provider, fx.userTwoToken.publicKey)).to.equal(
      "316060279"
    );
    expect(await tokenAmount(provider, fx.epochTokenVault)).to.equal("1");

    await waitForSlot(provider, finalized.claimDeadlineSlot.toNumber() + 1);
    const sweepSignature = await program.methods
      .sweepEpoch({ epochId: fx.epochId })
      .accountsStrict({
        oracle: oracle.publicKey,
        config,
        epoch: fx.epoch,
        epochTokenVault: fx.epochTokenVault,
        tokenReserveVault,
        mintAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([oracle])
      .rpc();
    await provider.connection.confirmTransaction(sweepSignature, "confirmed");

    const swept = await program.account.epoch.fetch(fx.epoch);
    expect(swept.status).to.deep.equal({ swept: {} });
    expect(await tokenAmount(provider, fx.epochTokenVault)).to.equal("0");
    expect(await tokenAmount(provider, tokenReserveVault)).to.equal("1");
  });

  it("rejects invalid quality factors and duplicate claims", async () => {
    const fx = await createEpochFixture();
    await openEpoch(fx, 10_000_000);
    await waitForSlot(provider, 2);
    await finalizeEpoch(fx, 100);

    const claim = pda([
      "claim",
      u64(fx.epochId),
      fx.userOne.publicKey.toBuffer(),
    ]);
    await expectRejected(
      createClaim(fx, claim, fx.userOne.publicKey, 50, QUALITY_PPM_SCALE + 1),
      "InvalidQualityFactor"
    );
    await createClaim(fx, claim, fx.userOne.publicKey, 50, QUALITY_PPM_SCALE);
    await expectRejected(
      createClaim(fx, claim, fx.userOne.publicKey, 50, QUALITY_PPM_SCALE),
      "already in use"
    );
  });

  async function createEpochFixture() {
    const epochId = new BN(nextEpochId++);
    const epoch = pda(["epoch", u64(epochId)]);
    const epochTokenVault = pda(["epoch_vault", u64(epochId)]);
    const userOne = web3.Keypair.generate();
    const userTwo = web3.Keypair.generate();
    const userOneToken = web3.Keypair.generate();
    const userTwoToken = web3.Keypair.generate();

    await createTokenAccount(
      provider,
      userOneToken,
      userOne.publicKey,
      mint.publicKey
    );
    await createTokenAccount(
      provider,
      userTwoToken,
      userTwo.publicKey,
      mint.publicKey
    );

    return {
      epochId,
      epoch,
      epochTokenVault,
      userOne,
      userTwo,
      userOneToken,
      userTwoToken,
    };
  }

  async function openEpoch(
    fx: Awaited<ReturnType<typeof createEpochFixture>>,
    supportBudget: number
  ) {
    await program.methods
      .openEpoch({
        epochId: fx.epochId,
        startSlot: new BN(0),
        endSlot: new BN(1),
        supportBudgetLamports: new BN(supportBudget),
      })
      .accountsStrict({
        oracle: oracle.publicKey,
        funder: payer.publicKey,
        config,
        epoch: fx.epoch,
        epochTokenVault: fx.epochTokenVault,
        mintAuthority,
        rewardMint: mint.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([oracle])
      .rpc();
  }

  async function finalizeEpoch(
    fx: Awaited<ReturnType<typeof createEpochFixture>>,
    totalRewardWeight: number
  ) {
    await program.methods
      .finalizeEpoch({ totalRewardWeight: new BN(totalRewardWeight) })
      .accountsStrict({
        oracle: oracle.publicKey,
        refundTarget: payer.publicKey,
        config,
        epoch: fx.epoch,
        epochTokenVault: fx.epochTokenVault,
        rewardMint: mint.publicKey,
        mintAuthority,
        solReserve,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([oracle])
      .rpc();
  }

  async function createClaim(
    fx: Awaited<ReturnType<typeof createEpochFixture>>,
    claim: web3.PublicKey,
    user: web3.PublicKey,
    bandwidthUnits: number,
    qualityFactorPpm: number
  ) {
    await program.methods
      .createClaim({
        epochId: fx.epochId,
        user,
        bandwidthUnits: new BN(bandwidthUnits),
        qualityFactorPpm: new BN(qualityFactorPpm),
      })
      .accountsStrict({
        oracle: oracle.publicKey,
        payer: payer.publicKey,
        config,
        epoch: fx.epoch,
        claim,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([oracle])
      .rpc();
  }

  async function claimReward(
    fx: Awaited<ReturnType<typeof createEpochFixture>>,
    claim: web3.PublicKey,
    user: web3.Keypair,
    userTokenAccount: web3.PublicKey
  ) {
    const signature = await program.methods
      .claimReward({ epochId: fx.epochId })
      .accountsStrict({
        user: user.publicKey,
        config,
        epoch: fx.epoch,
        claim,
        epochTokenVault: fx.epochTokenVault,
        userTokenAccount,
        mintAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
    await provider.connection.confirmTransaction(signature, "confirmed");
  }
});

function pda(seeds: Array<string | Buffer>): web3.PublicKey {
  return web3.PublicKey.findProgramAddressSync(
    seeds.map((seed) => (typeof seed === "string" ? Buffer.from(seed) : seed)),
    PROGRAM_ID
  )[0];
}

function u64(value: BN): Buffer {
  const out = Buffer.alloc(8);
  out.writeBigUInt64LE(BigInt(value.toString()));
  return out;
}

async function createTokenAccount(
  provider: AnchorProvider,
  tokenAccount: web3.Keypair,
  owner: web3.PublicKey,
  mint: web3.PublicKey
) {
  const rent = await provider.connection.getMinimumBalanceForRentExemption(
    TOKEN_ACCOUNT_LEN
  );
  const tx = new web3.Transaction().add(
    web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: tokenAccount.publicKey,
      lamports: rent,
      space: TOKEN_ACCOUNT_LEN,
      programId: TOKEN_PROGRAM_ID,
    }),
    new web3.TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: tokenAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: mint, isSigner: false, isWritable: false },
      ],
      data: Buffer.concat([Buffer.from([18]), owner.toBuffer()]),
    })
  );
  await provider.sendAndConfirm(tx, [tokenAccount]);
}

async function tokenAmount(
  provider: AnchorProvider,
  tokenAccount: web3.PublicKey
): Promise<string> {
  const account = await provider.connection.getAccountInfo(
    tokenAccount,
    "confirmed"
  );
  if (!account)
    throw new Error(`missing token account ${tokenAccount.toBase58()}`);
  return account.data.readBigUInt64LE(64).toString();
}

async function waitForSlot(provider: AnchorProvider, targetSlot: number) {
  while ((await provider.connection.getSlot("confirmed")) < targetSlot) {
    const tx = new web3.Transaction().add(
      web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: provider.wallet.publicKey,
        lamports: 1,
      })
    );
    await provider.sendAndConfirm(tx, []);
  }
}

async function expectRejected(promise: Promise<unknown>, expected: string) {
  try {
    await promise;
  } catch (error) {
    expect(String(error)).to.contain(expected);
    return;
  }
  throw new Error(`expected rejection containing ${expected}`);
}

import {
  address,
  type Address,
  type ReadonlyUint8Array,
  type TransactionSigner,
} from "@solana/kit";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { getClaimRewardInstructionAsync } from "@pcn-client/instructions/claimReward";
import { getCreateClaimInstructionAsync } from "@pcn-client/instructions/createClaim";
import { getFinalizeEpochInstructionAsync } from "@pcn-client/instructions/finalizeEpoch";
import { getInitializeConfigInstructionAsync } from "@pcn-client/instructions/initializeConfig";
import { getOpenEpochInstructionAsync } from "@pcn-client/instructions/openEpoch";
import { getSweepEpochInstructionAsync } from "@pcn-client/instructions/sweepEpoch";
import { getUpdateConfigInstructionAsync } from "@pcn-client/instructions/updateConfig";
import { PCN_PROGRAM_ID, TOKEN_DECIMALS } from "./config";
import {
  parseDecimalUnits,
  parseUnsignedInteger,
  shortenAddress,
} from "./format";
import {
  findClaimPda,
  findConfigPda,
  findEpochPda,
  findEpochVaultPda,
  findMintAuthorityPda,
} from "./pdas";
import type {
  ActionKind,
  PreparedTransaction,
  ProtocolSnapshot,
} from "./types";

type FormValues = Record<string, string>;
const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new PublicKey(
  "BPFLoaderUpgradeab1e11111111111111111111111"
);
type KitInstruction = {
  programAddress: Address;
  accounts: readonly { address: Address; role: number; signer?: unknown }[];
  data: ReadonlyUint8Array;
};

function toAddress(value: PublicKey | string) {
  return address(typeof value === "string" ? value : value.toBase58());
}

function toSigner(value: PublicKey): TransactionSigner {
  return {
    address: toAddress(value),
    signTransactions: async (transactions) => transactions,
  } as TransactionSigner;
}

function toWeb3Instruction(instruction: KitInstruction) {
  return new TransactionInstruction({
    programId: new PublicKey(instruction.programAddress),
    keys: instruction.accounts.map((account) => ({
      pubkey: new PublicKey(account.address),
      isSigner: account.role >= 2,
      isWritable: account.role === 1 || account.role === 3,
    })),
    data: Buffer.from(instruction.data),
  });
}

function publicKey(value: string, label: string) {
  try {
    return new PublicKey(value.trim());
  } catch {
    throw new Error(`${label} is not a valid Solana address.`);
  }
}

function requireConfig(snapshot: ProtocolSnapshot) {
  if (!snapshot.config) {
    throw new Error("Initialize the PCN config before using this action.");
  }
  return snapshot.config;
}

function requireRole(
  wallet: PublicKey,
  expected: string,
  role: "admin" | "oracle"
) {
  if (wallet.toBase58() !== expected) {
    throw new Error(`The connected wallet is not the configured ${role}.`);
  }
}

function curveFromForm(values: FormValues) {
  return {
    maxEpochMint: parseDecimalUnits(
      values.maxEpochMint,
      TOKEN_DECIMALS,
      "Max epoch mint"
    ),
    emissionMultiplierPpm: parseUnsignedInteger(
      values.emissionMultiplierPpm,
      "Emission multiplier"
    ),
    saturationUnits: parseUnsignedInteger(
      values.saturationUnits,
      "Saturation units"
    ),
    historyMinted: parseDecimalUnits(
      values.historyMinted,
      TOKEN_DECIMALS,
      "History minted"
    ),
    targetSupportLamportsPerToken: parseUnsignedInteger(
      values.targetSupportLamportsPerToken,
      "Support lamports per token"
    ),
    maxSupply: parseDecimalUnits(
      values.maxSupply,
      TOKEN_DECIMALS,
      "Max supply"
    ),
  };
}

function performanceWeightsFromForm(values: FormValues) {
  return {
    uptimePpm: parseUnsignedInteger(values.uptimeWeightPpm, "Uptime weight"),
    bandwidthPpm: parseUnsignedInteger(
      values.bandwidthWeightPpm,
      "Bandwidth weight"
    ),
    fulfilmentRatePpm: parseUnsignedInteger(
      values.fulfilmentRateWeightPpm,
      "Fulfilment-rate weight"
    ),
    questScorePpm: parseUnsignedInteger(
      values.questScoreWeightPpm,
      "Quest-score weight"
    ),
  };
}

function baseReview(
  wallet: PublicKey,
  action: string,
  description: string,
  instructions: TransactionInstruction[],
  summary: PreparedTransaction["summary"],
  extraSigners: Keypair[] = []
): PreparedTransaction {
  return {
    action,
    description,
    instructions,
    extraSigners,
    feePayer: wallet,
    summary: [
      { label: "Fee payer", value: shortenAddress(wallet.toBase58(), 6) },
      { label: "Program", value: shortenAddress(PCN_PROGRAM_ID.toBase58(), 6) },
      ...summary,
    ],
  };
}

export async function prepareTransaction(input: {
  action: ActionKind;
  connection: Connection;
  snapshot: ProtocolSnapshot;
  values: FormValues;
  wallet: PublicKey;
}): Promise<PreparedTransaction> {
  const { action: kind, connection, snapshot, values, wallet } = input;
  const signer = toSigner(wallet);

  if (kind === "initialize") {
    if (snapshot.config) {
      throw new Error("The PCN config account is already initialized.");
    }
    const rewardMint = Keypair.generate();
    const admin = publicKey(values.admin, "Admin");
    const oracle = publicKey(values.oracle, "Oracle");
    const [programData] = PublicKey.findProgramAddressSync(
      [PCN_PROGRAM_ID.toBuffer()],
      BPF_LOADER_UPGRADEABLE_PROGRAM_ID
    );
    const instruction = await getInitializeConfigInstructionAsync({
      payer: signer,
      program: toAddress(PCN_PROGRAM_ID),
      programData: toAddress(programData),
      rewardMint: toSigner(rewardMint.publicKey),
      admin: toAddress(admin),
      oracle: toAddress(oracle),
      claimWindowSlots: parseUnsignedInteger(
        values.claimWindowSlots,
        "Claim window slots"
      ),
      curve: curveFromForm(values),
      performanceWeights: performanceWeightsFromForm(values),
    });
    return baseReview(
      wallet,
      "Initialize protocol",
      "Creates the PCN config, mint, SOL reserve, and token reserve accounts.",
      [toWeb3Instruction(instruction)],
      [
        { label: "Admin", value: shortenAddress(admin.toBase58(), 6) },
        { label: "Oracle", value: shortenAddress(oracle.toBase58(), 6) },
        {
          label: "New mint",
          value: shortenAddress(rewardMint.publicKey.toBase58(), 6),
        },
      ],
      [rewardMint]
    );
  }

  const config = requireConfig(snapshot);
  const configPda = findConfigPda();

  if (kind === "update") {
    requireRole(wallet, config.admin, "admin");
    const oracle = publicKey(values.oracle, "Oracle");
    const instruction = await getUpdateConfigInstructionAsync({
      admin: signer,
      config: toAddress(configPda),
      oracle: toAddress(oracle),
      claimWindowSlots: parseUnsignedInteger(
        values.claimWindowSlots,
        "Claim window slots"
      ),
      curve: curveFromForm(values),
      performanceWeights: performanceWeightsFromForm(values),
    });
    return baseReview(
      wallet,
      "Update protocol settings",
      "Replaces the configured oracle, claim window, and scarcity curve.",
      [toWeb3Instruction(instruction)],
      [{ label: "Next oracle", value: shortenAddress(oracle.toBase58(), 6) }]
    );
  }

  if (kind === "open") {
    requireRole(wallet, config.oracle, "oracle");
    const epochId = parseUnsignedInteger(values.epochId, "Epoch ID");
    const budget = parseDecimalUnits(values.supportSol, 9, "Support budget");
    const instruction = await getOpenEpochInstructionAsync({
      oracle: signer,
      funder: signer,
      config: toAddress(configPda),
      epoch: toAddress(findEpochPda(epochId)),
      epochTokenVault: toAddress(findEpochVaultPda(epochId)),
      mintAuthority: toAddress(findMintAuthorityPda()),
      rewardMint: toAddress(config.rewardMint),
      systemProgram: toAddress(SystemProgram.programId),
      tokenProgram: toAddress(TOKEN_PROGRAM_ID),
      epochId,
      startSlot: parseUnsignedInteger(values.startSlot, "Start slot"),
      endSlot: parseUnsignedInteger(values.endSlot, "End slot"),
      supportBudgetLamports: budget,
    });
    return baseReview(
      wallet,
      `Open epoch ${epochId}`,
      "Escrows the support budget and opens a measurement window.",
      [toWeb3Instruction(instruction)],
      [
        { label: "Epoch", value: epochId.toString() },
        { label: "Support", value: `${values.supportSol} SOL` },
      ]
    );
  }

  if (kind === "finalize") {
    requireRole(wallet, config.oracle, "oracle");
    const epochId = parseUnsignedInteger(values.epochId, "Epoch ID");
    const epoch = snapshot.epochs.find((item) => item.epochId === epochId);
    if (!epoch) {
      throw new Error(
        `Epoch ${epochId} was not found in the current snapshot.`
      );
    }
    const instruction = await getFinalizeEpochInstructionAsync({
      oracle: signer,
      config: toAddress(configPda),
      epoch: toAddress(findEpochPda(epochId)),
      supportFunder: toAddress(epoch.supportFunder),
      epochTokenVault: toAddress(findEpochVaultPda(epochId)),
      rewardMint: toAddress(config.rewardMint),
      mintAuthority: toAddress(findMintAuthorityPda()),
      solReserve: toAddress(config.solReserve),
      tokenProgram: toAddress(TOKEN_PROGRAM_ID),
      systemProgram: toAddress(SystemProgram.programId),
      totalRewardWeight: parseUnsignedInteger(
        values.totalRewardWeight,
        "Total reward weight"
      ),
    });
    return baseReview(
      wallet,
      `Finalize epoch ${epochId}`,
      "Computes the supported reward pool, mints it, locks consumed SOL, and refunds excess support.",
      [toWeb3Instruction(instruction)],
      [
        { label: "Epoch", value: epochId.toString() },
        { label: "Total weight", value: values.totalRewardWeight },
        {
          label: "Support funder",
          value: shortenAddress(epoch.supportFunder, 6),
        },
      ]
    );
  }

  if (kind === "create-claim") {
    requireRole(wallet, config.oracle, "oracle");
    const epochId = parseUnsignedInteger(values.epochId, "Epoch ID");
    const user = publicKey(values.user, "Recipient");
    const instruction = await getCreateClaimInstructionAsync({
      oracle: signer,
      payer: signer,
      config: toAddress(configPda),
      epoch: toAddress(findEpochPda(epochId)),
      claim: toAddress(findClaimPda(epochId, user)),
      systemProgram: toAddress(SystemProgram.programId),
      epochId,
      user: toAddress(user),
      performance: {
        uptimePpm: parseUnsignedInteger(values.uptimePpm, "Uptime score"),
        bandwidthPpm: parseUnsignedInteger(
          values.bandwidthPpm,
          "Bandwidth score"
        ),
        fulfilmentRatePpm: parseUnsignedInteger(
          values.fulfilmentRatePpm,
          "Fulfilment-rate score"
        ),
        questScorePpm: parseUnsignedInteger(
          values.questScorePpm,
          "Quest score"
        ),
      },
    });
    return baseReview(
      wallet,
      `Create epoch ${epochId} claim`,
      "Records the oracle-verified four-metric performance score for one user.",
      [toWeb3Instruction(instruction)],
      [
        { label: "Recipient", value: shortenAddress(user.toBase58(), 6) },
        { label: "Uptime", value: `${values.uptimePpm} ppm` },
        { label: "Bandwidth", value: `${values.bandwidthPpm} ppm` },
      ]
    );
  }

  if (kind === "claim") {
    const epochId = parseUnsignedInteger(values.epochId, "Epoch ID");
    const epoch = snapshot.epochs.find((item) => item.epochId === epochId);
    if (!epoch) {
      throw new Error(
        `Epoch ${epochId} was not found in the current snapshot.`
      );
    }
    const claimPda = findClaimPda(epochId, wallet);
    const userTokenAccount = await getAssociatedTokenAddress(
      new PublicKey(config.rewardMint),
      wallet,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const instructions: TransactionInstruction[] = [];
    if (!(await connection.getAccountInfo(userTokenAccount, "confirmed"))) {
      instructions.push(
        createAssociatedTokenAccountInstruction(
          wallet,
          userTokenAccount,
          wallet,
          new PublicKey(config.rewardMint),
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        )
      );
    }
    const instruction = await getClaimRewardInstructionAsync({
      user: signer,
      config: toAddress(configPda),
      epoch: toAddress(findEpochPda(epochId)),
      claim: toAddress(claimPda),
      epochTokenVault: toAddress(epoch.epochTokenVault),
      userTokenAccount: toAddress(userTokenAccount),
      mintAuthority: toAddress(findMintAuthorityPda()),
      tokenProgram: toAddress(TOKEN_PROGRAM_ID),
      epochId,
    });
    instructions.push(toWeb3Instruction(instruction));
    return baseReview(
      wallet,
      `Claim epoch ${epochId} reward`,
      "Transfers the allocated PCN reward into your token account.",
      instructions,
      [
        { label: "Epoch", value: epochId.toString() },
        {
          label: "Token account",
          value: shortenAddress(userTokenAccount.toBase58(), 6),
        },
      ]
    );
  }

  requireRole(wallet, config.oracle, "oracle");
  const epochId = parseUnsignedInteger(values.epochId, "Epoch ID");
  const epoch = snapshot.epochs.find((item) => item.epochId === epochId);
  if (!epoch) {
    throw new Error(`Epoch ${epochId} was not found in the current snapshot.`);
  }
  const instruction = await getSweepEpochInstructionAsync({
    oracle: signer,
    config: toAddress(configPda),
    epoch: toAddress(findEpochPda(epochId)),
    epochTokenVault: toAddress(epoch.epochTokenVault),
    tokenReserveVault: toAddress(config.tokenReserveVault),
    mintAuthority: toAddress(findMintAuthorityPda()),
    tokenProgram: toAddress(TOKEN_PROGRAM_ID),
    epochId,
  });
  return baseReview(
    wallet,
    `Sweep epoch ${epochId}`,
    "Moves unclaimed PCN into the protocol reserve and permanently closes the epoch claim path.",
    [toWeb3Instruction(instruction)],
    [{ label: "Epoch", value: epochId.toString() }]
  );
}

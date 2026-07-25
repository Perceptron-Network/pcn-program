import assert from "node:assert/strict";
import test from "node:test";
import { EpochStatus } from "@pcn-client/types/epochStatus";
import { Keypair, type Connection } from "@solana/web3.js";
import { prepareTransaction } from "./transactions";
import type { ActionKind, ProtocolSnapshot } from "./types";

const wallet = Keypair.generate().publicKey;
const rewardMint = Keypair.generate().publicKey;
const configAddress = Keypair.generate().publicKey.toBase58();
const epochVault = Keypair.generate().publicKey.toBase58();

const snapshot: ProtocolSnapshot = {
  slot: 10_000n,
  config: {
    address: configAddress,
    admin: wallet.toBase58(),
    oracle: wallet.toBase58(),
    rewardMint: rewardMint.toBase58(),
    solReserve: Keypair.generate().publicKey.toBase58(),
    tokenReserveVault: Keypair.generate().publicKey.toBase58(),
    lifetimeCurveMintedAmount: 1_000_000_000n,
    claimWindowSlots: 1_000n,
    curve: {
      maxEpochMint: 100_000_000_000n,
      emissionMultiplierPpm: 1_000_000n,
      saturationUnits: 1_000n,
      historyMinted: 1_000_000_000_000n,
      targetSupportLamportsPerToken: 50_000n,
      maxSupply: 10_000_000_000_000n,
    },
    performanceWeights: {
      uptimePpm: 250_000n,
      bandwidthPpm: 250_000n,
      fulfilmentRatePpm: 250_000n,
      questScorePpm: 250_000n,
    },
  },
  epochs: [
    {
      address: Keypair.generate().publicKey.toBase58(),
      epochId: 7n,
      status: EpochStatus.Finalized,
      startSlot: 8_000n,
      endSlot: 9_000n,
      supportBudgetLamports: 1_000_000_000n,
      consumedSupportLamports: 500_000_000n,
      totalRewardWeight: 1_000n,
      rewardPoolAmount: 500_000_000_000n,
      allocatedAmount: 100_000_000_000n,
      claimedAmount: 0n,
      claimDeadlineSlot: 11_000n,
      epochTokenVault: epochVault,
      performanceWeights: {
        uptimePpm: 250_000n,
        bandwidthPpm: 250_000n,
        fulfilmentRatePpm: 250_000n,
        questScorePpm: 250_000n,
      },
    },
    {
      address: Keypair.generate().publicKey.toBase58(),
      epochId: 8n,
      status: EpochStatus.Open,
      startSlot: 10_000n,
      endSlot: 11_000n,
      supportBudgetLamports: 1_000_000_000n,
      consumedSupportLamports: 0n,
      totalRewardWeight: 0n,
      rewardPoolAmount: 0n,
      allocatedAmount: 0n,
      claimedAmount: 0n,
      claimDeadlineSlot: 0n,
      epochTokenVault: Keypair.generate().publicKey.toBase58(),
      performanceWeights: {
        uptimePpm: 250_000n,
        bandwidthPpm: 250_000n,
        fulfilmentRatePpm: 250_000n,
        questScorePpm: 250_000n,
      },
    },
  ],
  claims: [],
  walletClaims: [],
  reserveLamports: 500_000_000n,
  walletLamports: 2_000_000_000n,
  fetchedAt: Date.now(),
  source: "live",
};

const values = {
  admin: wallet.toBase58(),
  oracle: wallet.toBase58(),
  claimWindowSlots: "1000",
  maxEpochMint: "100",
  emissionMultiplierPpm: "1000000",
  saturationUnits: "1000",
  historyMinted: "1000",
  targetSupportLamportsPerToken: "50000",
  maxSupply: "10000",
  uptimeWeightPpm: "250000",
  bandwidthWeightPpm: "250000",
  fulfilmentRateWeightPpm: "250000",
  questScoreWeightPpm: "250000",
  epochId: "7",
  startSlot: "10000",
  endSlot: "11000",
  supportSol: "1",
  totalRewardWeight: "1000",
  refundTarget: wallet.toBase58(),
  user: wallet.toBase58(),
  uptimePpm: "950000",
  bandwidthPpm: "900000",
  fulfilmentRatePpm: "925000",
  questScorePpm: "975000",
};

const connection = {
  async getAccountInfo() {
    return null;
  },
} as unknown as Connection;

test("constructs all seven PCN instruction flows from the Codama client", async () => {
  const initialize = await prepareTransaction({
    action: "initialize",
    connection,
    snapshot: { ...snapshot, config: null },
    values,
    wallet,
  });
  assert.equal(initialize.instructions.length, 1);
  assert.equal(initialize.extraSigners.length, 1);

  const expectedInstructionCounts: Partial<Record<ActionKind, number>> = {
    update: 1,
    open: 1,
    finalize: 1,
    "create-claim": 1,
    claim: 2,
    sweep: 1,
  };

  for (const [action, expectedCount] of Object.entries(
    expectedInstructionCounts,
  ) as Array<[ActionKind, number]>) {
    const actionValues = {
      ...values,
      epochId: action === "finalize" ? "8" : "7",
    };
    const prepared = await prepareTransaction({
      action,
      connection,
      snapshot,
      values: actionValues,
      wallet,
    });
    assert.equal(
      prepared.instructions.length,
      expectedCount,
      `${action} instruction count`,
    );
    assert.equal(prepared.feePayer.toBase58(), wallet.toBase58());
  }
});

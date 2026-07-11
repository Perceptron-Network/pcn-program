import type { Address } from "@solana/kit";
import { Connection, PublicKey } from "@solana/web3.js";
import {
  CONFIG_DISCRIMINATOR,
  getConfigDecoder,
} from "@pcn-client/accounts/config";
import {
  EPOCH_DISCRIMINATOR,
  getEpochDecoder,
} from "@pcn-client/accounts/epoch";
import {
  CLAIM_DISCRIMINATOR,
  getClaimDecoder,
} from "@pcn-client/accounts/claim";
import { EpochStatus } from "@pcn-client/types/epochStatus";
import { PCN_PROGRAM_ID } from "./config";
import { findConfigPda } from "./pdas";
import type {
  ClaimView,
  ConfigView,
  EpochView,
  ProtocolSnapshot,
} from "./types";

function matchesDiscriminator(
  data: ArrayLike<number>,
  expected: ArrayLike<number>,
) {
  return (
    data.length >= expected.length &&
    Array.from(expected).every((value, index) => data[index] === value)
  );
}

function toConfigView(address: PublicKey, bytes: Uint8Array): ConfigView {
  const data = getConfigDecoder().decode(bytes);
  return {
    address: address.toBase58(),
    admin: data.admin,
    oracle: data.oracle,
    rewardMint: data.rewardMint,
    solReserve: data.solReserve,
    tokenReserveVault: data.tokenReserveVault,
    lifetimeCurveMintedAmount: data.lifetimeCurveMintedAmount,
    claimWindowSlots: data.claimWindowSlots,
    curve: data.curve,
  };
}

function toEpochView(address: PublicKey, bytes: Uint8Array): EpochView {
  const data = getEpochDecoder().decode(bytes);
  return {
    address: address.toBase58(),
    epochId: data.epochId,
    status: data.status,
    startSlot: data.startSlot,
    endSlot: data.endSlot,
    supportBudgetLamports: data.supportBudgetLamports,
    consumedSupportLamports: data.consumedSupportLamports,
    totalRewardWeight: data.totalRewardWeight,
    rewardPoolAmount: data.rewardPoolAmount,
    allocatedAmount: data.allocatedAmount,
    claimedAmount: data.claimedAmount,
    claimDeadlineSlot: data.claimDeadlineSlot,
    epochTokenVault: data.epochTokenVault,
  };
}

function toClaimView(address: PublicKey, bytes: Uint8Array): ClaimView {
  const data = getClaimDecoder().decode(bytes);
  return {
    address: address.toBase58(),
    epoch: data.epoch,
    epochId: data.epochId,
    user: data.user,
    bandwidthUnits: data.bandwidthUnits,
    qualityFactorPpm: data.qualityFactorPpm,
    rewardWeight: data.rewardWeight,
    rewardAmount: data.rewardAmount,
    claimed: data.claimed,
  };
}

function addressValue(value: string): Address {
  return value as Address;
}

export function createDemoSnapshot(walletAddress?: string): ProtocolSnapshot {
  const configAddress = findConfigPda().toBase58();
  const oracle = "7kPCNoraclewJi1hKqT9yQ8hJvX56kB9YtW8PcnDemo111";
  const user = walletAddress || "8dPCNuserUi2cz5D8cQrFfJtB1nRpkZY96PcnDemo1111";
  return {
    slot: 348_290_414n,
    config: {
      address: configAddress,
      admin: "3aPCNadminJLwQeN1R9QYdF7DkZBq19GsXPcnDemo1111",
      oracle,
      rewardMint: "5mPCNmintSgBgV3KpjB8Hf5cYrWPcnDemo1111111111",
      solReserve: "4sPCNreserveh3hQ2wKxPcnDemo11111111111111111",
      tokenReserveVault: "9vPCNreserveX5m3gPcnDemo111111111111111111",
      lifetimeCurveMintedAmount: 18_420_500_000_000n,
      claimWindowSlots: 216_000n,
      curve: {
        maxEpochMint: 2_500_000_000_000n,
        saturationUnits: 25_000_000n,
        historyMinted: 100_000_000_000_000n,
        targetSupportLamportsPerToken: 50_000n,
        maxSupply: 1_000_000_000_000_000n,
      },
    },
    epochs: [
      {
        address: "EpPCN004Fq7qY5PcnDemo1111111111111111111111",
        epochId: 42n,
        status: EpochStatus.Open,
        startSlot: 348_210_000n,
        endSlot: 348_426_000n,
        supportBudgetLamports: 1_400_000_000n,
        consumedSupportLamports: 0n,
        totalRewardWeight: 0n,
        rewardPoolAmount: 0n,
        allocatedAmount: 0n,
        claimedAmount: 0n,
        claimDeadlineSlot: 0n,
        epochTokenVault: "EvPCN004Fq7qY5PcnDemo1111111111111111111111",
      },
      {
        address: "EpPCN003Fq7qY5PcnDemo1111111111111111111111",
        epochId: 41n,
        status: EpochStatus.Finalized,
        startSlot: 347_994_000n,
        endSlot: 348_210_000n,
        supportBudgetLamports: 1_100_000_000n,
        consumedSupportLamports: 936_000_000n,
        totalRewardWeight: 19_900_000n,
        rewardPoolAmount: 1_872_000_000_000n,
        allocatedAmount: 1_704_500_000_000n,
        claimedAmount: 1_221_800_000_000n,
        claimDeadlineSlot: 348_426_000n,
        epochTokenVault: "EvPCN003Fq7qY5PcnDemo1111111111111111111111",
      },
      {
        address: "EpPCN002Fq7qY5PcnDemo1111111111111111111111",
        epochId: 40n,
        status: EpochStatus.Swept,
        startSlot: 347_778_000n,
        endSlot: 347_994_000n,
        supportBudgetLamports: 900_000_000n,
        consumedSupportLamports: 811_000_000n,
        totalRewardWeight: 17_340_000n,
        rewardPoolAmount: 1_622_000_000_000n,
        allocatedAmount: 1_590_000_000_000n,
        claimedAmount: 1_522_000_000_000n,
        claimDeadlineSlot: 348_210_000n,
        epochTokenVault: "EvPCN002Fq7qY5PcnDemo1111111111111111111111",
      },
    ],
    claims: [],
    walletClaims: [
      {
        address: "ClPCN41Fq7qY5PcnDemo11111111111111111111111",
        epoch: "EpPCN003Fq7qY5PcnDemo1111111111111111111111",
        epochId: 41n,
        user,
        bandwidthUnits: 4_820_000n,
        qualityFactorPpm: 972_000n,
        rewardWeight: 4_685_040n,
        rewardAmount: 440_651_812_060n,
        claimed: false,
      },
    ],
    reserveLamports: 12_482_000_000n,
    walletLamports: walletAddress ? 4_820_000_000n : null,
    fetchedAt: Date.now(),
    source: "demo",
  };
}

export async function loadProtocolSnapshot(
  connection: Connection,
  walletAddress?: string,
): Promise<ProtocolSnapshot> {
  const walletKey = walletAddress ? new PublicKey(walletAddress) : null;
  const [slot, programAccounts, walletLamports] = await Promise.all([
    connection.getSlot("confirmed"),
    connection.getProgramAccounts(PCN_PROGRAM_ID, { commitment: "confirmed" }),
    walletKey ? connection.getBalance(walletKey, "confirmed") : null,
  ]);

  let config: ConfigView | null = null;
  const epochs: EpochView[] = [];
  const claims: ClaimView[] = [];

  for (const account of programAccounts) {
    const bytes = new Uint8Array(account.account.data);
    try {
      if (matchesDiscriminator(bytes, CONFIG_DISCRIMINATOR)) {
        config = toConfigView(account.pubkey, bytes);
      } else if (matchesDiscriminator(bytes, EPOCH_DISCRIMINATOR)) {
        epochs.push(toEpochView(account.pubkey, bytes));
      } else if (matchesDiscriminator(bytes, CLAIM_DISCRIMINATOR)) {
        claims.push(toClaimView(account.pubkey, bytes));
      }
    } catch (error) {
      console.warn(
        "Ignored malformed PCN account",
        account.pubkey.toBase58(),
        error,
      );
    }
  }

  epochs.sort((left, right) => (left.epochId > right.epochId ? -1 : 1));
  claims.sort((left, right) => (left.epochId > right.epochId ? -1 : 1));

  const reserveLamports = config
    ? await connection.getBalance(new PublicKey(config.solReserve), "confirmed")
    : 0;

  return {
    slot: BigInt(slot),
    config,
    epochs,
    claims,
    walletClaims: walletAddress
      ? claims.filter((claim) => claim.user === walletAddress)
      : [],
    reserveLamports: BigInt(reserveLamports),
    walletLamports: walletLamports === null ? null : BigInt(walletLamports),
    fetchedAt: Date.now(),
    source: "live",
  };
}

export const asAddress = addressValue;

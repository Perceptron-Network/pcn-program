import type {
  PublicKey,
  TransactionInstruction,
  Keypair,
} from "@solana/web3.js";
import type {
  CurveParams,
  EpochStatus,
  PerformanceMetrics,
  PerformanceWeights,
} from "@pcn-client/types";

export type ConfigView = {
  address: string;
  admin: string;
  oracle: string;
  rewardMint: string;
  solReserve: string;
  tokenReserveVault: string;
  lifetimeCurveMintedAmount: bigint;
  claimWindowSlots: bigint;
  curve: CurveParams;
  performanceWeights: PerformanceWeights;
};

export type EpochView = {
  address: string;
  epochId: bigint;
  status: EpochStatus;
  startSlot: bigint;
  endSlot: bigint;
  supportBudgetLamports: bigint;
  consumedSupportLamports: bigint;
  totalRewardWeight: bigint;
  rewardPoolAmount: bigint;
  allocatedAmount: bigint;
  claimedAmount: bigint;
  claimDeadlineSlot: bigint;
  epochTokenVault: string;
  performanceWeights: PerformanceWeights;
};

export type ClaimView = {
  address: string;
  epoch: string;
  epochId: bigint;
  user: string;
  performance: PerformanceMetrics;
  rewardWeight: bigint;
  rewardAmount: bigint;
  claimed: boolean;
};

export type ProtocolSnapshot = {
  slot: bigint;
  config: ConfigView | null;
  epochs: EpochView[];
  claims: ClaimView[];
  walletClaims: ClaimView[];
  reserveLamports: bigint;
  walletLamports: bigint | null;
  fetchedAt: number;
  source: "live" | "demo";
};

export type PreparedTransaction = {
  action: string;
  description: string;
  instructions: TransactionInstruction[];
  extraSigners: Keypair[];
  feePayer: PublicKey;
  summary: Array<{ label: string; value: string }>;
};

export type ActionKind =
  | "initialize"
  | "update"
  | "open"
  | "finalize"
  | "create-claim"
  | "claim"
  | "sweep";

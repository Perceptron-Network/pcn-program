import { PublicKey } from "@solana/web3.js";

const environment = (import.meta as ImportMeta & { env?: ImportMetaEnv }).env;

export const PCN_PROGRAM_ID = new PublicKey(
  "86oGodFG8DfLYHAgYwUPCNQzaods7Auxz9cnU7XzWipt"
);

export const RPC_URL =
  environment?.VITE_RPC_URL?.trim() || "http://127.0.0.1:8899";

export const CLUSTER = environment?.VITE_CLUSTER || "localnet";

export const DEMO_MODE =
  environment?.VITE_DEMO_MODE === "true" ||
  (typeof window !== "undefined" &&
    new URLSearchParams(window.location.search).get("demo") === "1");

/**
 * Slot time the default claim window below was sized against, in milliseconds.
 *
 * Mainnet no longer runs at 400ms: it is 350ms as of epoch 1020, 300ms from
 * epoch 1024, and the target is 200ms. Anything derived from this constant is
 * therefore an assumption, not a measurement.
 */
export const ASSUMED_MS_PER_SLOT = 400;

/**
 * Default `claim_window_slots` / demo epoch length used to pre-fill the admin
 * forms, expressed in slots.
 *
 * 216_000 slots x {@link ASSUMED_MS_PER_SLOT} == 86_400_000ms == 24h of
 * intended wall-clock claim window. At the current 350ms it is ~21h, at 300ms
 * ~18h, and at the 200ms target ~12h.
 *
 * This number is written ON-CHAIN and gates `create_claim` and `sweep_epoch`,
 * so it is deliberately NOT auto-adjusted here: shortening the real claim
 * period changes protocol economics and is an operator decision. It stays
 * updatable on-chain through `update_config`, so retuning it does not need a
 * redeploy.
 */
export const DEFAULT_CLAIM_WINDOW_SLOTS = 216_000n;

export const TOKEN_DECIMALS = 6;
export const TOKEN_BASE_UNITS = 1_000_000n;
export const LAMPORTS_PER_SOL = 1_000_000_000n;

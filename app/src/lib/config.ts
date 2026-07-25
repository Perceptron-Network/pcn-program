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

export const TOKEN_DECIMALS = 9;
export const TOKEN_BASE_UNITS = 1_000_000_000n;
export const LAMPORTS_PER_SOL = 1_000_000_000n;

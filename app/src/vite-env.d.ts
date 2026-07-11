/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_CLUSTER?: "localnet" | "devnet";
  readonly VITE_DEMO_MODE?: string;
  readonly VITE_RPC_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

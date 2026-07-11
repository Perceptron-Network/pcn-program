import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { useStandardWalletAdapters } from "@solana/wallet-standard-wallet-adapter-react";
import type { ReactNode } from "react";
import { RPC_URL } from "@/lib/config";

function WalletAdapters({ children }: { children: ReactNode }) {
  const adapters = useStandardWalletAdapters([]);

  return (
    <WalletProvider wallets={adapters} autoConnect={false}>
      {children}
    </WalletProvider>
  );
}

export function AppWalletProvider({ children }: { children: ReactNode }) {
  return (
    <ConnectionProvider endpoint={RPC_URL}>
      <WalletAdapters>{children}</WalletAdapters>
    </ConnectionProvider>
  );
}

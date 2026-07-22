import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { DEMO_MODE } from "@/lib/config";
import { createDemoSnapshot, loadProtocolSnapshot } from "@/lib/protocol";
import type { ProtocolSnapshot } from "@/lib/types";

type ProtocolState = {
  snapshot: ProtocolSnapshot | null;
  loading: boolean;
  error: string | null;
};

export function useProtocolSnapshot() {
  const { connection } = useConnection();
  const { publicKey } = useWallet();
  const walletAddress = publicKey?.toBase58();
  const requestId = useRef(0);
  const [state, setState] = useState<ProtocolState>(() => ({
    snapshot: DEMO_MODE ? createDemoSnapshot(walletAddress) : null,
    loading: !DEMO_MODE,
    error: null,
  }));

  const refresh = useCallback(async () => {
    const currentRequest = ++requestId.current;
    if (DEMO_MODE) {
      setState({
        snapshot: createDemoSnapshot(walletAddress),
        loading: false,
        error: null,
      });
      return;
    }

    setState((current) => ({ ...current, loading: true, error: null }));
    try {
      const snapshot = await loadProtocolSnapshot(connection, walletAddress);
      if (currentRequest === requestId.current) {
        setState({ snapshot, loading: false, error: null });
      }
    } catch (cause) {
      if (currentRequest === requestId.current) {
        setState((current) => ({
          ...current,
          loading: false,
          error:
            cause instanceof Error
              ? cause.message
              : "Unable to read the PCN program.",
        }));
      }
    }
  }, [connection, walletAddress]);

  useEffect(() => {
    void refresh();
    const interval = window.setInterval(() => void refresh(), 15_000);
    return () => window.clearInterval(interval);
  }, [refresh]);

  return { ...state, refresh };
}

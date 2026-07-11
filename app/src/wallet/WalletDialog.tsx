import { useWallet } from "@solana/wallet-adapter-react";
import { Check, ChevronRight, X } from "lucide-react";
import { useState } from "react";

export function WalletDialog({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const { connect, connecting, select, wallet, wallets } = useWallet();
  const [error, setError] = useState<string | null>(null);

  if (!open) {
    return null;
  }

  async function handleConnect() {
    setError(null);
    try {
      await connect();
      onClose();
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : "Wallet connection failed.",
      );
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="wallet-dialog modal-sheet"
        role="dialog"
        aria-modal="true"
        aria-labelledby="wallet-dialog-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <div>
            <span className="eyebrow">Wallet handshake</span>
            <h2 id="wallet-dialog-title">Choose a signer</h2>
          </div>
          <button className="icon-button" onClick={onClose} aria-label="Close">
            <X size={18} />
          </button>
        </header>

        <p className="modal-intro">
          Selection never triggers a connection. Choose a wallet, then confirm
          with the separate connect button below.
        </p>

        <div className="wallet-list">
          {wallets.length === 0 ? (
            <p className="empty-copy">
              No compatible browser wallets detected.
            </p>
          ) : (
            wallets.map((candidate) => {
              const selected = wallet?.adapter.name === candidate.adapter.name;
              return (
                <button
                  className={`wallet-choice ${selected ? "is-selected" : ""}`}
                  key={candidate.adapter.name}
                  onClick={() => select(candidate.adapter.name)}
                >
                  <img
                    src={candidate.adapter.icon}
                    alt=""
                    width={34}
                    height={34}
                  />
                  <span>
                    <strong>{candidate.adapter.name}</strong>
                    <small>{candidate.readyState}</small>
                  </span>
                  {selected ? <Check size={18} /> : <ChevronRight size={18} />}
                </button>
              );
            })
          )}
        </div>

        {error ? <p className="form-error">{error}</p> : null}
        <button
          className="button button-primary button-wide"
          disabled={!wallet || connecting}
          onClick={handleConnect}
        >
          {connecting ? "Connecting…" : "Connect selected wallet"}
        </button>
      </section>
    </div>
  );
}

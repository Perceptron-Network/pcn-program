import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { EpochStatus } from "@pcn-client/types/epochStatus";
import {
  ArrowDownRight,
  ArrowUpRight,
  CheckCircle2,
  CircleDot,
  Copy,
  ExternalLink,
  Menu,
  Radio,
  RefreshCw,
  ServerCrash,
  ShieldCheck,
  Wallet,
  X,
} from "lucide-react";
import { lazy, Suspense, useMemo, useState } from "react";
import { Transaction } from "@solana/web3.js";
import { TransactionReview } from "@/components/TransactionReview";
import { useProtocolSnapshot } from "@/hooks/useProtocolSnapshot";
import { CLUSTER, DEMO_MODE, PCN_PROGRAM_ID } from "@/lib/config";
import {
  formatInteger,
  formatPcn,
  formatPercent,
  formatSol,
  shortenAddress,
} from "@/lib/format";
import type { ActionKind, EpochView, PreparedTransaction } from "@/lib/types";
import { WalletDialog } from "@/wallet/WalletDialog";

const ActionPanel = lazy(() =>
  import("@/components/ActionPanel").then((module) => ({
    default: module.ActionPanel,
  })),
);

function epochStatusLabel(status: EpochStatus) {
  if (status === EpochStatus.Open) return "Measuring";
  if (status === EpochStatus.Finalized) return "Claiming";
  return "Swept";
}

function epochStatusClass(status: EpochStatus) {
  if (status === EpochStatus.Open) return "status-open";
  if (status === EpochStatus.Finalized) return "status-finalized";
  return "status-swept";
}

function progressPercent(value: bigint, total: bigint) {
  if (total <= 0n) return 0;
  return Math.min(100, Number((value * 10_000n) / total) / 100);
}

function copyText(value: string) {
  void navigator.clipboard.writeText(value);
}

function Header({ onWallet }: { onWallet: () => void }) {
  const { connected, disconnect, publicKey } = useWallet();
  const [menuOpen, setMenuOpen] = useState(false);
  return (
    <header className="site-header">
      <a href="#top" className="brand" aria-label="PCN home">
        <span className="brand-mark" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className="brand-type">
          <strong>PCN</strong>
          <small>Physical Connectivity Network</small>
        </span>
      </a>

      <nav className={menuOpen ? "nav-links is-open" : "nav-links"}>
        <a href="#epochs" onClick={() => setMenuOpen(false)}>
          Epochs
        </a>
        <a href="#claims" onClick={() => setMenuOpen(false)}>
          My rewards
        </a>
        <a href="#actions" onClick={() => setMenuOpen(false)}>
          Operate
        </a>
        <a
          href={`https://explorer.solana.com/address/${PCN_PROGRAM_ID.toBase58()}?cluster=custom`}
          target="_blank"
          rel="noreferrer"
        >
          Explorer <ExternalLink size={13} />
        </a>
      </nav>

      <div className="header-actions">
        <span className="network-badge">
          <i /> {CLUSTER}
        </span>
        {connected && publicKey ? (
          <button
            className="wallet-button is-connected"
            onClick={() => void disconnect()}
          >
            <span>{shortenAddress(publicKey.toBase58())}</span>
            <small>Disconnect</small>
          </button>
        ) : (
          <button className="wallet-button" onClick={onWallet}>
            <Wallet size={16} /> Connect wallet
          </button>
        )}
        <button
          className="mobile-menu-button"
          onClick={() => setMenuOpen((value) => !value)}
          aria-label="Toggle navigation"
        >
          {menuOpen ? <X size={20} /> : <Menu size={20} />}
        </button>
      </div>
    </header>
  );
}

function EpochRow({ epoch, slot }: { epoch: EpochView; slot: bigint }) {
  const poolProgress = progressPercent(
    epoch.claimedAmount,
    epoch.rewardPoolAmount,
  );
  const activeProgress = progressPercent(
    slot - epoch.startSlot,
    epoch.endSlot - epoch.startSlot,
  );
  return (
    <article className="epoch-row">
      <div className="epoch-identity">
        <span className="epoch-number">
          E/{epoch.epochId.toString().padStart(3, "0")}
        </span>
        <span className={`status-pill ${epochStatusClass(epoch.status)}`}>
          <i /> {epochStatusLabel(epoch.status)}
        </span>
      </div>
      <div className="epoch-measure">
        <small>
          {epoch.status === EpochStatus.Open
            ? "Slot progress"
            : "Rewards claimed"}
        </small>
        <strong>
          {epoch.status === EpochStatus.Open
            ? `${Math.max(0, activeProgress).toFixed(1)}%`
            : `${poolProgress.toFixed(1)}%`}
        </strong>
        <span className="progress-track">
          <i
            style={{
              width: `${epoch.status === EpochStatus.Open ? activeProgress : poolProgress}%`,
            }}
          />
        </span>
      </div>
      <div className="epoch-value">
        <small>Reward pool</small>
        <strong>
          {epoch.rewardPoolAmount
            ? formatPcn(epoch.rewardPoolAmount)
            : "Pending"}
        </strong>
        <span>
          {epoch.rewardPoolAmount
            ? "PCN"
            : `${formatSol(epoch.supportBudgetLamports)} SOL support`}
        </span>
      </div>
      <div className="epoch-value">
        <small>Bandwidth weight</small>
        <strong>{formatInteger(epoch.totalRewardWeight)}</strong>
        <span>quality adjusted</span>
      </div>
      <button
        className="copy-button"
        onClick={() => copyText(epoch.address)}
        aria-label="Copy epoch address"
      >
        <Copy size={15} />
      </button>
    </article>
  );
}

export function App() {
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const { snapshot, loading, error, refresh } = useProtocolSnapshot();
  const [walletOpen, setWalletOpen] = useState(false);
  const [prepared, setPrepared] = useState<PreparedTransaction | null>(null);
  const [transactionError, setTransactionError] = useState<string | null>(null);
  const [transactionPending, setTransactionPending] = useState(false);
  const [lastSignature, setLastSignature] = useState<string | null>(null);

  const claimable = useMemo(
    () => snapshot?.walletClaims.filter((claim) => !claim.claimed) || [],
    [snapshot?.walletClaims],
  );
  const claimableAmount = claimable.reduce(
    (total, claim) => total + claim.rewardAmount,
    0n,
  );

  async function handlePrepare(
    action: ActionKind,
    values: Record<string, string>,
  ) {
    if (!publicKey)
      throw new Error("Connect a wallet before preparing an instruction.");
    if (!snapshot)
      throw new Error("Wait for a protocol snapshot before continuing.");
    if (snapshot.source === "demo") {
      throw new Error(
        "Preview data is read-only. Remove ?demo=1 to use live localnet data.",
      );
    }
    const { prepareTransaction } = await import("@/lib/transactions");
    const next = await prepareTransaction({
      action,
      connection,
      snapshot,
      values,
      wallet: publicKey,
    });
    setTransactionError(null);
    setPrepared(next);
    return next;
  }

  async function submitPrepared() {
    if (!prepared || !publicKey) return;
    setTransactionPending(true);
    setTransactionError(null);
    try {
      const latest = await connection.getLatestBlockhash("confirmed");
      const transaction = new Transaction({
        feePayer: prepared.feePayer,
        blockhash: latest.blockhash,
        lastValidBlockHeight: latest.lastValidBlockHeight,
      }).add(...prepared.instructions);
      if (prepared.extraSigners.length > 0) {
        transaction.partialSign(...prepared.extraSigners);
      }

      const simulation = await connection.simulateTransaction(transaction);
      if (simulation.value.err) {
        const detail =
          simulation.value.logs?.slice(-3).join(" · ") ||
          JSON.stringify(simulation.value.err);
        throw new Error(`Simulation failed: ${detail}`);
      }

      const signature = await sendTransaction(transaction, connection, {
        signers: prepared.extraSigners,
        preflightCommitment: "confirmed",
        skipPreflight: false,
        maxRetries: 3,
      });
      const confirmation = await connection.confirmTransaction(
        { signature, ...latest },
        "confirmed",
      );
      if (confirmation.value.err) {
        throw new Error(
          `Transaction failed: ${JSON.stringify(confirmation.value.err)}`,
        );
      }
      setLastSignature(signature);
      setPrepared(null);
      await refresh();
    } catch (cause) {
      setTransactionError(
        cause instanceof Error ? cause.message : "Transaction failed.",
      );
    } finally {
      setTransactionPending(false);
    }
  }

  const config = snapshot?.config;
  const maxSupply = config?.curve.maxSupply || 0n;
  const minted = config?.lifetimeCurveMintedAmount || 0n;
  const supplyUsed = progressPercent(minted, maxSupply);

  return (
    <div className="app-shell" id="top">
      <Header onWallet={() => setWalletOpen(true)} />

      <main>
        <section className="hero-section">
          <div className="hero-grid-lines" aria-hidden="true" />
          <div className="hero-copy">
            <span className="eyebrow hero-eyebrow">
              <Radio size={14} /> Solana bandwidth rewards / v1
            </span>
            <h1>
              Bandwidth becomes
              <em>public infrastructure.</em>
            </h1>
            <p>
              PCN turns verified network contribution into scarce, SOL-supported
              rewards. One clear console for operators and participants.
            </p>
            <div className="hero-actions">
              <a className="button button-primary" href="#claims">
                View my rewards <ArrowDownRight size={18} />
              </a>
              <a className="button button-outline" href="#epochs">
                Inspect epochs
              </a>
            </div>
          </div>

          <aside className="telemetry-card">
            <header>
              <span>Network telemetry</span>
              <i className={error ? "signal-dot signal-error" : "signal-dot"} />
            </header>
            <div className="telemetry-primary">
              <small>Current slot</small>
              <strong>{snapshot ? formatInteger(snapshot.slot) : "—"}</strong>
              <span>
                {DEMO_MODE ? "preview feed" : loading ? "syncing" : "confirmed"}
              </span>
            </div>
            <div className="telemetry-wave" aria-hidden="true">
              {Array.from({ length: 28 }, (_, index) => (
                <i
                  key={index}
                  style={{ height: `${18 + ((index * 17) % 58)}%` }}
                />
              ))}
            </div>
            <dl>
              <div>
                <dt>Program</dt>
                <dd>{shortenAddress(PCN_PROGRAM_ID.toBase58(), 6)}</dd>
              </div>
              <div>
                <dt>RPC</dt>
                <dd>{error ? "Offline" : "Healthy"}</dd>
              </div>
              <div>
                <dt>State</dt>
                <dd>{config ? "Initialized" : "Awaiting config"}</dd>
              </div>
            </dl>
          </aside>
        </section>

        {DEMO_MODE ? (
          <div className="demo-banner">
            <CircleDot size={15} /> Preview data is active. Transactions are
            disabled.
          </div>
        ) : null}

        {error ? (
          <section className="connection-error">
            <ServerCrash size={24} />
            <div>
              <strong>RPC connection unavailable</strong>
              <span>{error}</span>
            </div>
            <button
              className="button button-outline"
              onClick={() => void refresh()}
            >
              Retry
            </button>
          </section>
        ) : null}

        <section className="metrics-strip" aria-label="Protocol metrics">
          <article>
            <span className="metric-index">01</span>
            <div>
              <small>Lifetime minted</small>
              <strong>{config ? formatPcn(minted) : "—"}</strong>
              <span>PCN</span>
            </div>
            <ArrowUpRight size={18} />
          </article>
          <article>
            <span className="metric-index">02</span>
            <div>
              <small>SOL held in reserve</small>
              <strong>
                {snapshot ? formatSol(snapshot.reserveLamports) : "—"}
              </strong>
              <span>SOL</span>
            </div>
            <ShieldCheck size={18} />
          </article>
          <article>
            <span className="metric-index">03</span>
            <div>
              <small>Epochs observed</small>
              <strong>
                {snapshot ? formatInteger(snapshot.epochs.length) : "—"}
              </strong>
              <span>on this cluster</span>
            </div>
            <Radio size={18} />
          </article>
          <article>
            <span className="metric-index">04</span>
            <div>
              <small>Curve supply used</small>
              <strong>{supplyUsed.toFixed(2)}%</strong>
              <span>
                {config
                  ? `${formatPcn(maxSupply - minted)} PCN remains`
                  : "no config"}
              </span>
            </div>
            <ArrowDownRight size={18} />
          </article>
        </section>

        <section className="content-section epoch-section" id="epochs">
          <div className="section-heading">
            <div>
              <span className="eyebrow">Epoch ledger</span>
              <h2>Reward cycle, in public</h2>
            </div>
            <button
              className="refresh-button"
              onClick={() => void refresh()}
              disabled={loading}
            >
              <RefreshCw size={15} className={loading ? "spin" : ""} /> Refresh
              snapshot
            </button>
          </div>
          <div className="epoch-table">
            {snapshot?.epochs.length ? (
              snapshot.epochs.map((epoch) => (
                <EpochRow
                  key={epoch.address}
                  epoch={epoch}
                  slot={snapshot.slot}
                />
              ))
            ) : (
              <div className="empty-state">
                <RadioTowerIllustration />
                <strong>No epochs yet</strong>
                <span>
                  The configured oracle can open the first measurement window.
                </span>
              </div>
            )}
          </div>
        </section>

        <section className="content-section rewards-section" id="claims">
          <div className="rewards-hero">
            <span className="eyebrow">Participant rewards</span>
            <h2>
              Your bandwidth.
              <br />
              Your proof. Your PCN.
            </h2>
            <p>
              Claims are created from permissioned measurements and remain
              visible on-chain from allocation through redemption.
            </p>
          </div>
          <div className="claim-console">
            <header>
              <div>
                <small>Available to claim</small>
                <strong>
                  {publicKey ? formatPcn(claimableAmount) : "—"}
                  <span> PCN</span>
                </strong>
              </div>
              <span className="claim-count">
                {claimable.length} open claim{claimable.length === 1 ? "" : "s"}
              </span>
            </header>
            {!publicKey ? (
              <button
                className="connect-claim"
                onClick={() => setWalletOpen(true)}
              >
                <Wallet size={20} />
                <span>
                  <strong>Connect to inspect your claim PDAs</strong>
                  <small>Connection begins only after your confirmation.</small>
                </span>
                <ArrowUpRight size={18} />
              </button>
            ) : claimable.length ? (
              <div className="claim-list">
                {claimable.map((claim) => (
                  <article key={claim.address}>
                    <span className="claim-epoch">
                      Epoch {claim.epochId.toString()}
                    </span>
                    <div>
                      <small>Bandwidth units</small>
                      <strong>{formatInteger(claim.bandwidthUnits)}</strong>
                    </div>
                    <div>
                      <small>Quality</small>
                      <strong>{formatPercent(claim.qualityFactorPpm)}</strong>
                    </div>
                    <div>
                      <small>Reward</small>
                      <strong>{formatPcn(claim.rewardAmount)} PCN</strong>
                    </div>
                    <CheckCircle2 size={18} />
                  </article>
                ))}
              </div>
            ) : (
              <div className="claim-empty">
                <CircleDot size={20} />
                <span>
                  <strong>No open claims</strong>
                  <small>New oracle allocations will appear here.</small>
                </span>
              </div>
            )}
          </div>
        </section>

        {snapshot ? (
          <Suspense
            fallback={
              <section className="action-panel action-loading">
                Loading program controls…
              </section>
            }
          >
            <ActionPanel snapshot={snapshot} onPrepare={handlePrepare} />
          </Suspense>
        ) : null}

        <section className="curve-section">
          <div>
            <span className="eyebrow">Scarcity policy</span>
            <h2>
              Support is not decoration.
              <br />
              It is the emission ceiling.
            </h2>
          </div>
          <dl className="curve-list">
            <div>
              <dt>Maximum epoch mint</dt>
              <dd>
                {config ? `${formatPcn(config.curve.maxEpochMint)} PCN` : "—"}
              </dd>
            </div>
            <div>
              <dt>Saturation target</dt>
              <dd>
                {config ? formatInteger(config.curve.saturationUnits) : "—"}
              </dd>
            </div>
            <div>
              <dt>Support per token</dt>
              <dd>
                {config
                  ? `${formatInteger(config.curve.targetSupportLamportsPerToken)} lamports`
                  : "—"}
              </dd>
            </div>
            <div>
              <dt>Claim window</dt>
              <dd>
                {config
                  ? `${formatInteger(config.claimWindowSlots)} slots`
                  : "—"}
              </dd>
            </div>
          </dl>
        </section>
      </main>

      <footer className="site-footer">
        <div className="brand footer-brand">
          <span className="brand-mark">
            <i />
            <i />
            <i />
          </span>
          <span className="brand-type">
            <strong>PCN</strong>
            <small>Rewarding useful connectivity</small>
          </span>
        </div>
        <p>
          Permissioned measurement. Transparent settlement. Standard SPL Token.
        </p>
        <button
          className="program-address"
          onClick={() => copyText(PCN_PROGRAM_ID.toBase58())}
        >
          {shortenAddress(PCN_PROGRAM_ID.toBase58(), 8)} <Copy size={13} />
        </button>
      </footer>

      {lastSignature ? (
        <div className="success-toast">
          <CheckCircle2 size={18} />
          <span>
            <strong>Transaction confirmed</strong>
            <small>{shortenAddress(lastSignature, 8)}</small>
          </span>
          <button onClick={() => setLastSignature(null)} aria-label="Dismiss">
            <X size={15} />
          </button>
        </div>
      ) : null}
      <WalletDialog open={walletOpen} onClose={() => setWalletOpen(false)} />
      <TransactionReview
        prepared={prepared}
        cluster={CLUSTER}
        pending={transactionPending}
        error={transactionError}
        onClose={() => !transactionPending && setPrepared(null)}
        onSubmit={() => void submitPrepared()}
      />
    </div>
  );
}

function RadioTowerIllustration() {
  return (
    <span className="empty-illustration" aria-hidden="true">
      <i />
      <i />
      <i />
      <b />
    </span>
  );
}

import { useWallet } from "@solana/wallet-adapter-react";
import {
  ArrowLeft,
  ArrowRight,
  BadgeCheck,
  CircleDollarSign,
  Gauge,
  LockKeyhole,
  RadioTower,
  Settings2,
  UserRoundCheck,
} from "lucide-react";
import {
  useEffect,
  useState,
  useTransition,
  type ComponentType,
  type SVGProps,
} from "react";
import { formatDecimalUnits } from "@/lib/format";
import type {
  ActionKind,
  PreparedTransaction,
  ProtocolSnapshot,
} from "@/lib/types";

type ActionDefinition = {
  kind: ActionKind;
  label: string;
  detail: string;
  role: "Anyone" | "Admin" | "Oracle" | "User";
  Icon: ComponentType<SVGProps<SVGSVGElement>>;
};

const ACTIONS: ActionDefinition[] = [
  {
    kind: "initialize",
    label: "Initialize",
    detail: "Create protocol custody and reward mint",
    role: "Anyone",
    Icon: RadioTower,
  },
  {
    kind: "update",
    label: "Configure",
    detail: "Update oracle and scarcity curve",
    role: "Admin",
    Icon: Settings2,
  },
  {
    kind: "open",
    label: "Open epoch",
    detail: "Fund a bandwidth measurement window",
    role: "Oracle",
    Icon: Gauge,
  },
  {
    kind: "finalize",
    label: "Finalize",
    detail: "Mint the supported epoch reward pool",
    role: "Oracle",
    Icon: BadgeCheck,
  },
  {
    kind: "create-claim",
    label: "Create claim",
    detail: "Assign measured bandwidth rewards",
    role: "Oracle",
    Icon: UserRoundCheck,
  },
  {
    kind: "claim",
    label: "Claim PCN",
    detail: "Receive your allocated PCN reward",
    role: "User",
    Icon: CircleDollarSign,
  },
  {
    kind: "sweep",
    label: "Sweep epoch",
    detail: "Move expired rewards to reserve",
    role: "Oracle",
    Icon: LockKeyhole,
  },
];

type FieldDefinition = {
  key: string;
  label: string;
  hint?: string;
  placeholder?: string;
  type?: "text" | "number";
};

const CURVE_FIELDS: FieldDefinition[] = [
  { key: "maxEpochMint", label: "Max epoch mint", hint: "PCN" },
  { key: "saturationUnits", label: "Saturation units", hint: "weight units" },
  { key: "historyMinted", label: "History minted", hint: "PCN" },
  {
    key: "targetSupportLamportsPerToken",
    label: "Target support",
    hint: "lamports / PCN",
  },
  { key: "maxSupply", label: "Maximum supply", hint: "PCN" },
];

const ACTION_FIELDS: Record<ActionKind, FieldDefinition[]> = {
  initialize: [
    { key: "admin", label: "Admin address" },
    { key: "oracle", label: "Oracle address" },
    { key: "claimWindowSlots", label: "Claim window", hint: "slots" },
    ...CURVE_FIELDS,
  ],
  update: [
    { key: "oracle", label: "Oracle address" },
    { key: "claimWindowSlots", label: "Claim window", hint: "slots" },
    ...CURVE_FIELDS,
  ],
  open: [
    { key: "epochId", label: "Epoch ID" },
    { key: "startSlot", label: "Start slot" },
    { key: "endSlot", label: "End slot" },
    { key: "supportSol", label: "Support budget", hint: "SOL" },
  ],
  finalize: [
    { key: "epochId", label: "Epoch ID" },
    { key: "totalRewardWeight", label: "Total reward weight" },
    { key: "refundTarget", label: "Refund target" },
  ],
  "create-claim": [
    { key: "epochId", label: "Epoch ID" },
    { key: "user", label: "Recipient address" },
    { key: "bandwidthUnits", label: "Bandwidth units" },
    {
      key: "qualityFactorPpm",
      label: "Quality factor",
      hint: "0–1,000,000 ppm",
    },
  ],
  claim: [{ key: "epochId", label: "Epoch ID" }],
  sweep: [{ key: "epochId", label: "Epoch ID" }],
};

function toPcnInput(value: bigint) {
  return formatDecimalUnits(value, 1_000_000_000n, 9).replaceAll(",", "");
}

function defaultsFor(
  kind: ActionKind,
  snapshot: ProtocolSnapshot,
  walletAddress: string,
) {
  const config = snapshot.config;
  const latestEpoch = snapshot.epochs[0];
  const nextEpoch = latestEpoch ? latestEpoch.epochId + 1n : 1n;
  const finalizedEpoch = snapshot.epochs.find((epoch) => epoch.status === 1);
  const openEpoch = snapshot.epochs.find((epoch) => epoch.status === 0);
  const claim = snapshot.walletClaims.find((item) => !item.claimed);
  const defaultEpoch =
    kind === "finalize"
      ? openEpoch?.epochId
      : kind === "claim"
        ? claim?.epochId
        : finalizedEpoch?.epochId;

  return {
    admin: config?.admin || walletAddress,
    oracle: config?.oracle || walletAddress,
    claimWindowSlots: (config?.claimWindowSlots || 216_000n).toString(),
    maxEpochMint: toPcnInput(config?.curve.maxEpochMint || 2_500_000_000_000n),
    saturationUnits: (config?.curve.saturationUnits || 25_000_000n).toString(),
    historyMinted: toPcnInput(
      config?.curve.historyMinted || 100_000_000_000_000n,
    ),
    targetSupportLamportsPerToken: (
      config?.curve.targetSupportLamportsPerToken || 50_000n
    ).toString(),
    maxSupply: toPcnInput(config?.curve.maxSupply || 1_000_000_000_000_000n),
    epochId: (kind === "open"
      ? nextEpoch
      : defaultEpoch || nextEpoch
    ).toString(),
    startSlot: snapshot.slot.toString(),
    endSlot: (snapshot.slot + 216_000n).toString(),
    supportSol: "1",
    totalRewardWeight: "1000000",
    refundTarget: walletAddress,
    user: walletAddress,
    bandwidthUnits: "1000000",
    qualityFactorPpm: "1000000",
  };
}

export function ActionPanel({
  snapshot,
  onPrepare,
}: {
  snapshot: ProtocolSnapshot;
  onPrepare: (
    kind: ActionKind,
    values: Record<string, string>,
  ) => Promise<PreparedTransaction>;
}) {
  const { publicKey } = useWallet();
  const walletAddress = publicKey?.toBase58() || "";
  const [selected, setSelected] = useState<ActionDefinition | null>(null);
  const [values, setValues] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const [isPending, startTransition] = useTransition();

  useEffect(() => {
    if (selected) {
      setValues(defaultsFor(selected.kind, snapshot, walletAddress));
      setError(null);
    }
  }, [selected, snapshot, walletAddress]);

  function handleReview() {
    if (!selected) return;
    setError(null);
    startTransition(async () => {
      try {
        await onPrepare(selected.kind, values);
      } catch (cause) {
        setError(
          cause instanceof Error
            ? cause.message
            : "Unable to prepare transaction.",
        );
      }
    });
  }

  if (!selected) {
    return (
      <section className="action-panel" id="actions">
        <div className="section-heading action-heading">
          <div>
            <span className="eyebrow">Program controls</span>
            <h2>Operate the reward cycle</h2>
          </div>
          <span className="role-readout">
            {walletAddress ? "Signer detected" : "Connect to operate"}
          </span>
        </div>
        <div className="action-grid">
          {ACTIONS.map((action) => {
            const disabled =
              !walletAddress ||
              (action.kind === "initialize" && Boolean(snapshot.config)) ||
              (action.kind !== "initialize" && !snapshot.config);
            return (
              <button
                className="action-tile"
                key={action.kind}
                disabled={disabled}
                onClick={() => setSelected(action)}
              >
                <span className="action-icon">
                  <action.Icon width={21} height={21} />
                </span>
                <span className="action-copy">
                  <small>{action.role}</small>
                  <strong>{action.label}</strong>
                  <span>{action.detail}</span>
                </span>
                <ArrowRight size={18} className="action-arrow" />
              </button>
            );
          })}
        </div>
        <p className="panel-note">
          Every action opens a review step. Nothing connects, signs, or sends in
          the background.
        </p>
      </section>
    );
  }

  return (
    <section className="action-panel action-form-panel" id="actions">
      <button className="back-button" onClick={() => setSelected(null)}>
        <ArrowLeft size={16} /> All program controls
      </button>
      <div className="section-heading action-heading">
        <div>
          <span className="eyebrow">{selected.role} instruction</span>
          <h2>{selected.label}</h2>
          <p>{selected.detail}</p>
        </div>
        <span className="action-icon action-icon-large">
          <selected.Icon width={26} height={26} />
        </span>
      </div>
      <div className="form-grid">
        {ACTION_FIELDS[selected.kind].map((field) => (
          <label className="field" key={field.key}>
            <span>
              {field.label}
              {field.hint ? <small>{field.hint}</small> : null}
            </span>
            <input
              type={field.type || "text"}
              value={values[field.key] || ""}
              placeholder={field.placeholder}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  [field.key]: event.target.value,
                }))
              }
            />
          </label>
        ))}
      </div>
      {error ? <p className="form-error">{error}</p> : null}
      <button
        className="button button-primary review-button"
        onClick={handleReview}
        disabled={isPending}
      >
        {isPending ? "Preparing…" : "Review transaction"}
        <ArrowRight size={18} />
      </button>
    </section>
  );
}

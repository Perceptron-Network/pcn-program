import { AlertTriangle, Check, LoaderCircle, X } from "lucide-react";
import type { PreparedTransaction } from "@/lib/types";

export function TransactionReview({
  prepared,
  cluster,
  pending,
  error,
  onClose,
  onSubmit,
}: {
  prepared: PreparedTransaction | null;
  cluster: string;
  pending: boolean;
  error: string | null;
  onClose: () => void;
  onSubmit: () => void;
}) {
  if (!prepared) return null;

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="transaction-review modal-sheet"
        role="dialog"
        aria-modal="true"
        aria-labelledby="transaction-review-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <div>
            <span className="eyebrow">Final signer checkpoint</span>
            <h2 id="transaction-review-title">{prepared.action}</h2>
          </div>
          <button
            className="icon-button"
            onClick={onClose}
            aria-label="Close"
            disabled={pending}
          >
            <X size={18} />
          </button>
        </header>

        <p className="modal-intro">{prepared.description}</p>
        <div className="review-warning">
          <AlertTriangle size={18} />
          <span>
            This will simulate against <strong>{cluster}</strong> first. Your
            wallet opens only after simulation succeeds.
          </span>
        </div>

        <dl className="review-list">
          {prepared.summary.map((item) => (
            <div key={item.label}>
              <dt>{item.label}</dt>
              <dd>{item.value}</dd>
            </div>
          ))}
          <div>
            <dt>Instructions</dt>
            <dd>{prepared.instructions.length}</dd>
          </div>
          <div>
            <dt>Additional signers</dt>
            <dd>{prepared.extraSigners.length}</dd>
          </div>
        </dl>

        {error ? <p className="form-error">{error}</p> : null}
        <div className="review-actions">
          <button
            className="button button-quiet"
            onClick={onClose}
            disabled={pending}
          >
            Cancel
          </button>
          <button
            className="button button-primary"
            onClick={onSubmit}
            disabled={pending}
          >
            {pending ? (
              <>
                <LoaderCircle size={17} className="spin" /> Simulating / sending
              </>
            ) : (
              <>
                <Check size={17} /> Simulate &amp; submit
              </>
            )}
          </button>
        </div>
      </section>
    </div>
  );
}

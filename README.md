# PCN Program

A small Anchor program for permissioned bandwidth rewards on Solana.

The program mints PCN tokens to users based on bandwidth measurements submitted by a trusted oracle. Emissions are capped by a scarcity/support curve and by the SOL support budget deposited for each epoch.

## How It Works

1. **Initialize config**
   - The program upgrade authority creates the program config and selects the admin.
   - The program creates a 9-decimal SPL reward mint, a mint-authority PDA, a SOL reserve, and a token reserve vault.

2. **Open an epoch**
   - The configured oracle opens an epoch.
   - A funder deposits a SOL support budget into the epoch account.

3. **Finalize the epoch**
   - After the epoch ends, the oracle submits the total reward weight.
   - The program computes the reward pool from:
     - total performance reward weight
     - lifetime minted supply
     - remaining max supply
     - available SOL support
   - PCN tokens are minted into the epoch token vault.
   - Used SOL stays in the program SOL reserve; unused SOL is refunded.

4. **Create claims**
   - The oracle submits four normalized performance metrics for each user.
   - The program applies the epoch's snapshotted governance weights and stores the resulting reward weight and amount.

5. **Claim rewards**
   - Users claim from the epoch token vault into their own SPL token account.
   - Each claim can be redeemed once.

6. **Sweep leftovers**
   - After the claim window expires, the oracle can sweep unclaimed epoch tokens into the token reserve vault.

## Main Accounts

- `Config`: admin, oracle, reward mint, reserves, claim window, curve parameters, and governance performance weights.
- `Epoch`: epoch status, snapshotted performance weights, oracle-supplied total score, SOL budget, reward pool, claim deadline, and token vault.
- `Claim`: one user's four performance metrics and computed reward weight for one epoch.

## Emission Multiplier And Performance Scoring

`CurveParams.emission_multiplier_ppm` is the protocol-set `R` term in the
scarcity curve. It must be between `1` and `1_000_000`; `1_000_000` means
`R = 1`. The existing admin may update it by submitting a complete validated
curve through `update_config`.

The oracle verifies and submits normalized uptime, bandwidth, fulfilment-rate,
and quest scores in parts per million. The program applies the governance
weights snapshotted when the epoch opens:

`P = (w1*Uptime + w2*Bandwidth + w3*FulfilmentRate + w4*QuestScore) / 1_000_000`

Each metric is constrained to `0..=1_000_000`, and the four weights must sum to
`1_000_000`. The oracle continues to submit the aggregate epoch weight during
finalization; the program uses each computed `P` for proportional claim
allocation against that total.

## Key Commands

```bash
cargo test
cargo fmt
anchor test
yarn test:ts
```

Use `cargo test` for Rust unit and LiteSVM tests. Use `anchor test` when checking the full Anchor/TypeScript flow.

## Web app

The Vite/React interface lives in `app/` and consumes the generated Codama
TypeScript client directly. Start it with `yarn app:dev`; see `app/README.md` for
local-validator and read-only preview setup.

## Generated clients

The Anchor IDL is the source of truth for checked-in Codama clients:

- Rust: `codama/codama-rust`
- TypeScript: `codama/codama-ts`

Run `yarn gen-clients` after changing the program interface. It rebuilds the
IDL, regenerates and formats both clients, and verifies that a second generation
produces no drift. `anchor build` runs the same generation check through its
post-build hook; set `CODAMA_POST_BUILD_SKIP=1` only when a surrounding command
will regenerate the clients itself.

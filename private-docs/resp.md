# Hacken Remediation Response

## Scope confirmation

This remediation branch is based on the audited commit
`a47dbcdae999faa5b0703a1d30711c73e58b8dcf`. It contains exactly one commit for
each formal finding. Please confirm that configurable `R`, deterministic
fixed-point scarcity math, and test-only property/fuzz coverage are accepted as
in-scope remediation work.

## Formal findings remediated

1. **F-2026-18080 — unrestricted initialization.** `initialize_config` requires
   the payer to be the program's upgrade authority and verifies that the supplied
   ProgramData belongs to this program. No new authority system is introduced.
2. **F-2026-18078 — account allocation.** `Config`, `Epoch`, `Claim`,
   `CurveParams`, and nested serialized types derive `InitSpace`; account
   allocation is `8 + Type::INIT_SPACE` and is covered by serialization tests.
3. **F-2026-18079 — unused Rent accounts.** The Rent sysvar is removed from
   `initialize_config` and `open_epoch`, including their audited-base clients and
   tests.
4. **F-2026-18093 — floating-point scarcity math.** Production scarcity math is
   deterministic checked Q64.64 arithmetic with bounded range reduction,
   saturation, and integer exponential approximation. `libm` is no longer a
   direct production dependency. Boundary, differential, property, and
   long-running test-only fuzz coverage are included.
5. **F-2026-18096 — emission multiplier and scoring explanation.**
   `CurveParams.emission_multiplier_ppm` implements `0 < R <= 1`, with
   `1_000_000` representing `R = 1`. It is validated and can be changed through
   the existing admin-authorized full curve update. Performance scoring remains
   off-chain: the oracle submits its final scaled composite score as
   `bandwidth_units` with `quality_factor_ppm = 1_000_000`.

The multiplier is applied to the scarcity result before the existing SOL-support
and remaining-supply caps. No new update instruction, authority, or governance
mechanism is added.

## Current implementation explanations

- **SOL support:** the audited implementation caps each epoch by its deposited
  SOL support budget. Consumed support moves to the program reserve and unused
  support is refunded during finalization. This response documents that existing
  behavior; it does not propose a tokenomics redesign.
- **Epoch funding and refunds:** the audited implementation accepts a separate
  funder but does not persist the funder identity, and finalization accepts a
  caller-supplied refund target. Those behaviors are unchanged in this
  remediation branch.
- **Supply and minting:** the audited implementation mints epoch rewards through
  the program mint-authority PDA and enforces its configured tracked
  `max_supply`. This response does not claim that this is equivalent to a fixed
  genesis allocation.
- **Performance score:** the oracle is responsible for calculating the approved
  composite score and scaling off-chain. Canonical on-chain encoding uses that
  final value directly, with a quality factor of one million.

## Separately scoped whitepaper and product work

The following topics are deliberately excluded from this remediation branch:

- storing a funder or constraining the refund destination;
- redesigning supply or replacing minting with a pre-funded reserve;
- implementing claim, quest, or access-payment burns;
- binding the finalized denominator to an on-chain claim total;
- removing or redesigning SOL support;
- implementing score components or governance weights on-chain; and
- adding multisig, timelock, pause, emergency, or broader oracle controls.

If any of these changes are approved as product requirements, they should be
implemented on a separate branch and submitted for separate audit scope before
deployment.

## Deployment assumption

This remediation assumes no live `Config` account has been initialized. The new
`CurveParams` field changes the serialized account layout; if a live config
exists, a migration must be designed and pre-approved before these commits are
deployed.

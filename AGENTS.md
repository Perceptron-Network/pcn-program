# Repository Guidelines

## Project Structure & Module Organization

This is an Anchor workspace for the `pcn-program` Solana program. Program source lives in `programs/pcn-program/src/`. Entrypoints are wired from `lib.rs`, instruction handlers live under `src/instructions/<domain>/<instruction>.rs`, and each domain `mod.rs` should contain only `pub mod` / `pub use` declarations. Account state lives in `src/state/` with one account type per file, such as `config.rs`, `epoch.rs`, and `claim.rs`. Shared reward math is in `src/rewards.rs`; constants and errors are in `src/constants.rs` and `src/error.rs`.

Rust integration tests live in `programs/pcn-program/tests/`, including LiteSVM tests under `tests/litesvm/`. TypeScript Anchor tests live in root `tests/`. Deployment scaffolding is in `migrations/`, and `plan.md` records product/spec context.

## Build, Test, and Development Commands

- `cargo fmt`: format Rust code across the workspace.
- `cargo test`: run Rust unit tests plus LiteSVM integration tests.
- `yarn test:ts`: run TypeScript Anchor tests with `ts-mocha`.
- `yarn test`: same as `yarn test:ts`.
- `anchor test`: run the Anchor test script from `Anchor.toml`, currently `yarn test:ts && cargo test`.
- `yarn lint`: check Prettier formatting for JavaScript/TypeScript files.
- `yarn lint:fix`: apply Prettier formatting to JavaScript/TypeScript files.

## Coding Style & Naming Conventions

Use Rust 2021 style and `cargo fmt`. Keep instruction files named after their handlers, for example `open_epoch.rs` containing `open_epoch`, `OpenEpoch`, and `OpenEpochArgs`. Keep `mod.rs` files limited to imports/exports. For Anchor account allocation constants, use `pub const LEN: usize = 8 + size_of::<Self>();`. Prefer explicit account validation constraints and custom errors from `PcnError`.

TypeScript tests use CommonJS, Mocha, Chai, and ES2020 settings from `tsconfig.json`. Run Prettier before changing TS files.

## Testing Guidelines

Add Rust unit tests near pure logic, especially reward math. Add LiteSVM coverage for on-chain account behavior, authorization, validation, and token flows. Name validation test files by behavior, such as `claim_validation.rs` or `epoch_validation.rs`. For end-to-end TypeScript coverage, add cases under `tests/**/*.ts`.

Run `cargo test` before submitting Rust or program changes. Run `anchor test` when changes affect generated Anchor client behavior or TS flows.

## Commit & Pull Request Guidelines

The current git history is too small to infer a strict convention. Use concise, descriptive commit messages such as `split instruction modules` or `add claim validation tests`. Pull requests should describe the behavior change, list the commands run, and call out any account layout, PDA seed, token authority, or reward-curve changes. Link related issues or spec notes when applicable.

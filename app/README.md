# PCN app

The browser console reads the checked-in Codama TypeScript client directly from
`../codama/codama-ts`. It supports all seven v1 instructions and uses a separate
review/simulation checkpoint before opening a wallet transaction.

```bash
cp .env.example .env.local
yarn install
yarn dev
```

The safe default is a validator at `http://127.0.0.1:8899`. Use `?demo=1` for a
clearly labelled, read-only visual preview when no validator is running.

Checks:

```bash
yarn typecheck
yarn test
yarn prettier:check
yarn build
```

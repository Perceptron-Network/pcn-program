#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
IDL_PATH="${ROOT}/target/idl/pcn_program.json"

if [ ! -f "${IDL_PATH}" ]; then
  cat >&2 <<EOF
error: missing Anchor IDL at ${IDL_PATH}

Run:
  yarn gen-clients
EOF
  exit 1
fi

tmp_base="${TMPDIR:-/tmp}"
TMP_DIR="$(mktemp -d "${tmp_base%/}/pcn-codama-check.XXXXXX")"
trap 'rm -rf "${TMP_DIR}"' EXIT

RUST_DIR="${TMP_DIR}/codama-rust"
TS_DIR="${TMP_DIR}/codama-ts"

cd "${ROOT}"
"${ROOT}/node_modules/.bin/tsx" scripts/gen-client.ts \
  "${IDL_PATH}" "${RUST_DIR}" "${TS_DIR}"
bash scripts/format-codama.sh "${RUST_DIR}" "${TS_DIR}"

failed=0
if ! diff -ru "${ROOT}/codama/codama-rust" "${RUST_DIR}"; then
  echo "Codama Rust client drift detected. Run: yarn gen-clients" >&2
  failed=1
fi
if ! diff -ru "${ROOT}/codama/codama-ts" "${TS_DIR}"; then
  echo "Codama TypeScript client drift detected. Run: yarn gen-clients" >&2
  failed=1
fi

if [ "${failed}" -ne 0 ]; then
  exit 1
fi

echo "Codama clients are in sync with target/idl/pcn_program.json."

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
RUST_DIR="${1:-${ROOT}/codama/codama-rust}"
TS_DIR="${2:-${ROOT}/codama/codama-ts}"

if [ -d "${RUST_DIR}" ]; then
  find "${RUST_DIR}" -type f -name "*.rs" -exec rustfmt --edition 2021 {} +
fi

if [ -d "${TS_DIR}" ]; then
  "${ROOT}/node_modules/.bin/prettier" "${TS_DIR}/**/*.ts" -w --loglevel warn
fi

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "${CODAMA_POST_BUILD_SKIP:-0}" = "1" ]; then
  echo "Skipping Codama post-build generation."
  exit 0
fi

cd "${ROOT}"
yarn codama
yarn codama:format
yarn codama:check

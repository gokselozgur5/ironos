#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

TARGET="riscv64gc-unknown-none-elf"
KERNEL_DIR="${REPO_ROOT}/crates/ironos-kernel"

echo "[qemu-smoke] Building ironos-kernel for ${TARGET} in release mode..."
cargo build \
  --manifest-path "${KERNEL_DIR}/Cargo.toml" \
  --target "${TARGET}" \
  --release

KERNEL_ELF="${KERNEL_DIR}/target/${TARGET}/release/ironos-kernel"
if [[ ! -f "${KERNEL_ELF}" ]]; then
  KERNEL_ELF="${REPO_ROOT}/target/${TARGET}/release/ironos-kernel"
fi

if [[ ! -f "${KERNEL_ELF}" ]]; then
  echo "[qemu-smoke] ERROR: Kernel binary not found at ${KERNEL_ELF}" >&2
  exit 1
fi

echo "[qemu-smoke] Launching deterministic QEMU runner..."
python3 -u "${SCRIPT_DIR}/qemu_smoke.py" "${KERNEL_ELF}"

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

echo "[qemu-smoke] Launching QEMU virt machine..."
CMD_INPUT="help\nls\ncat motd\nwrite note hello-ironos\ncat note\nmem\nps\nticks\nhalt\n"
OUTPUT=$(printf '%b' "${CMD_INPUT}" | timeout 15 qemu-system-riscv64 \
  -machine virt \
  -bios default \
  -nographic \
  -kernel "${KERNEL_ELF}" 2>&1 || true)

echo "--- QEMU BOOT LOG START ---"
echo "${OUTPUT}"
echo "--- QEMU BOOT LOG END ---"

LINE1="[ironos] boot ok"
LINE2="[ironos] shell ready"
LINE3="ironos riscv64 kernel ready"
LINE4="hello-ironos"
LINE5="[ironos] halt"

for line in "${LINE1}" "${LINE2}" "${LINE3}" "${LINE4}" "${LINE5}"; do
  if ! echo "${OUTPUT}" | grep -qF "${line}"; then
    echo "[qemu-smoke] ERROR: Expected line missing: '${line}'" >&2
    exit 1
  fi
done

echo "[qemu-smoke] All boot smoke assertions PASSED."

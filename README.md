# ironos

From-scratch `no_std` Rust RISC-V 64 Mini OS Kernel with Sv39 paging, physical frame allocation, and QEMU boot verification.

## Layout

- `crates/ironos-lib`: Pure `no_std` kernel support library (frame allocator, ring buffer, Sv39 PTEs).
- `crates/ironos-kernel`: Standalone `riscv64gc-unknown-none-elf` bare-metal kernel binary.
- `scripts/qemu-smoke.sh`: QEMU virt machine automated smoke boot test.

## Building and Testing

```bash
# Host unit tests and linting
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

# Kernel bare-metal compilation
cargo build --manifest-path crates/ironos-kernel/Cargo.toml --target riscv64gc-unknown-none-elf --release
```

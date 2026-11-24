# ironos

From-scratch `no_std` Rust RISC-V 64 Mini OS Kernel with Sv39 paging, physical frame allocation, and QEMU boot verification.

## Layout

- `crates/ironos-lib`: Pure `no_std` kernel support library (frame allocator, ring buffer, Sv39 PTEs, RamFs, round-robin scheduler, shell parser).
- `crates/ironos-kernel`: Standalone `riscv64gc-unknown-none-elf` bare-metal kernel binary with S-mode trap vector, timer interrupts, and interactive serial shell.
- `scripts/qemu-smoke.sh`: QEMU virt machine automated smoke boot and interactive shell test.

## Shell Commands

- `help`: list available interactive shell commands
- `mem`: report RAM filesystem file count and storage usage
- `ps`: list registered scheduler tasks and runtime tick counts
- `ls`: list files in the in-memory RAM filesystem
- `cat <name>`: display contents of a stored file
- `write <name> <text>`: create or overwrite a file in RAM filesystem
- `ticks`: display cumulative timer interrupt tick count
- `halt`: clean kernel halt and SBI machine poweroff

## Building and Testing

```bash
# Host unit tests and linting
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

# Kernel bare-metal compilation
cargo build --manifest-path crates/ironos-kernel/Cargo.toml --target riscv64gc-unknown-none-elf --release
```

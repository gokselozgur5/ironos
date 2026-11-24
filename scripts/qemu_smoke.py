import subprocess
import sys
import time

EXPECTED_OUTPUTS = [
    "[ironos] boot ok",
    "[ironos] shell ready",
    "commands: help",
    "motd",
    "ironos riscv64 kernel ready",
    "hello-ironos",
    "ramfs:",
    "task 1:",
    "task 2:",
    "ticks:",
    "[ironos] halt",
]

COMMANDS = [
    "help",
    "ls",
    "cat motd",
    "write note hello-ironos",
    "cat note",
    "mem",
    "ps",
    "ticks",
    "halt",
]


def run_smoke(elf_path: str) -> None:
    cmd = [
        "qemu-system-riscv64",
        "-machine",
        "virt",
        "-bios",
        "default",
        "-nographic",
        "-kernel",
        elf_path,
    ]
    p = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    out = bytearray()
    start = time.time()

    def read_until(pat: bytes, timeout: float = 10.0) -> None:
        deadline = time.time() + timeout
        while time.time() < deadline:
            chunk = (
                p.stdout.read1(1024)
                if hasattr(p.stdout, "read1")
                else p.stdout.read(1)
            )
            if chunk:
                out.extend(chunk)
                if pat in out:
                    return
            elif p.poll() is not None:
                break
            time.sleep(0.01)
        raise TimeoutError(f"Missing {pat!r} in output")
    try:
        read_until(b"[ironos] shell ready")
        for c in COMMANDS:
            p.stdin.write(f"{c}\n".encode())
            p.stdin.flush()
            read_until(b"ironos> " if c != "halt" else b"[ironos] halt")
        p.wait(timeout=max(1.0, 15.0 - (time.time() - start)))
    finally:
        if p.poll() is None:
            p.kill()
        if p.stdout:
            out.extend(p.stdout.read())

    log = out.decode("utf-8", errors="replace")
    print(f"--- QEMU BOOT LOG START ---\n{log}--- QEMU BOOT LOG END ---")
    if p.returncode != 0:
        raise RuntimeError(f"QEMU exited with code {p.returncode}")

    for bad in ["unknown command:", "[ironos panic]", "[ironos trap]"]:
        if bad in log:
            raise AssertionError(f"Unexpected failure signature: {bad}")

    for req in EXPECTED_OUTPUTS:
        if req not in log:
            raise AssertionError(f"Expected output missing: {req}")

    print("[qemu-smoke] All boot smoke assertions PASSED.")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <kernel_elf>", file=sys.stderr)
        sys.exit(1)
    run_smoke(sys.argv[1])

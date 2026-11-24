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

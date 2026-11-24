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

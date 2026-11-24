// IronOS Web Hardware Emulator and RISC-V 64 Kernel Shell Simulator
// Strictly mirroring crates/ironos-lib (allocator, ramfs, sched, sv39)

class IronEmulator {
  constructor() {
    this.PAGE_SIZE = 4096;
    this.BASE_PADDR = 0x80400000;
    this.NUM_FRAMES = 64;
    this.bitmap = new Uint8Array(Math.ceil(this.NUM_FRAMES / 8));
    this.framesAllocated = 0;
    this.ramfs = new Map();
    this.ramfs.set('motd', 'ironos riscv64 kernel ready\n');
    this.tasks = [
      { id: 1, name: 'idle', state: 'Ready', ticks: 120 },
      { id: 2, name: 'shell', state: 'Running', ticks: 45 }
    ];
    this.ticks = 165;
    this.history = [];
    this.historyIdx = -1;
    this.listeners = [];
  }

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

  allocFrame() {
    for (let i = 0; i < this.NUM_FRAMES; i++) {
      const byteIdx = Math.floor(i / 8);
      const bitMask = 1 << (i % 8);
      if ((this.bitmap[byteIdx] & bitMask) === 0) {
        this.bitmap[byteIdx] |= bitMask;
        this.framesAllocated++;
        this.notifyChange();
        return this.BASE_PADDR + i * this.PAGE_SIZE;
      }
    }
    return null;
  }

  freeFrame(paddr) {
    if (paddr < this.BASE_PADDR) return false;
    const offset = paddr - this.BASE_PADDR;
    if (offset % this.PAGE_SIZE !== 0) return false;
    const frameIdx = offset / this.PAGE_SIZE;
    if (frameIdx >= this.NUM_FRAMES) return false;
    const byteIdx = Math.floor(frameIdx / 8);
    const bitMask = 1 << (frameIdx % 8);
    if ((this.bitmap[byteIdx] & bitMask) === 0) return false;
    this.bitmap[byteIdx] &= ~bitMask;
    this.framesAllocated--;
    this.notifyChange();
    return true;
  }

  decodeSv39(vaddrBig) {
    const vaddr = BigInt(vaddrBig);
    const signBit = (vaddr >> 38n) & 1n;
    const top25 = (vaddr >> 39n) & 0x1ffffffn;
    const isCanonical = signBit === 1n ? top25 === 0x1ffffffn : top25 === 0n;
    const vpn2 = Number((vaddr >> 30n) & 0x1ffn);
    const vpn1 = Number((vaddr >> 21n) & 0x1ffn);
    const vpn0 = Number((vaddr >> 12n) & 0x1ffn);
    const offset = Number(vaddr & 0xfffn);
    return {
      vaddr: '0x' + vaddr.toString(16).padStart(16, '0'),
      vpn2,
      vpn1,
      vpn0,
      offset: '0x' + offset.toString(16).padStart(3, '0'),
      isCanonical,
      isAligned: offset === 0
    };
  }

  notifyChange() {
    for (const cb of this.listeners) {
      cb();
    }
  }

  onChange(cb) {
    this.listeners.push(cb);
  }

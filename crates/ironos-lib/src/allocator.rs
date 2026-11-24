//! Fixed-size bitmap physical page frame allocator.

/// Standard page size on RISC-V (4 KiB).
pub const PAGE_SIZE: usize = 4096;

/// Bitmap-backed physical page frame allocator.
///
/// Manages `WORDS * 64` physical frames using a bit array of `u64` words.
/// Bit = 0 means free, Bit = 1 means allocated.
#[derive(Debug)]
pub struct FrameAllocator<const WORDS: usize> {
    base_paddr: usize,
    bitmap: [u64; WORDS],
    free_frames: usize,
}

impl<const WORDS: usize> FrameAllocator<WORDS> {
    /// Total number of page frames managed by this allocator.
    pub const TOTAL_FRAMES: usize = WORDS * 64;

    /// Creates a new frame allocator covering `TOTAL_FRAMES` from `base_paddr`.
    #[must_use]
    pub fn new(base_paddr: usize) -> Self {
        assert!(
            base_paddr.is_multiple_of(PAGE_SIZE),
            "base_paddr must be 4 KiB aligned"
        );
        Self {
            base_paddr,
            bitmap: [0u64; WORDS],
            free_frames: WORDS * 64,
        }
    }

    /// Returns base physical address managed by this allocator.
    #[must_use]
    pub const fn base_paddr(&self) -> usize {
        self.base_paddr
    }

    /// Returns count of currently free frames.
    #[must_use]
    pub const fn free_count(&self) -> usize {
        self.free_frames
    }

    /// Allocates one physical frame and returns its physical address.
    pub fn alloc_frame(&mut self) -> Option<usize> {
        if self.free_frames == 0 {
            return None;
        }

        for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
            if *word != u64::MAX {
                let bit_idx = (!*word).trailing_zeros() as usize;
                let frame_idx = word_idx * 64 + bit_idx;
                let paddr = frame_idx
                    .checked_mul(PAGE_SIZE)
                    .and_then(|off| self.base_paddr.checked_add(off))?;
                *word |= 1u64 << bit_idx;
                self.free_frames -= 1;
                return Some(paddr);
            }
        }
        None
    }

    /// Frees an allocated physical frame.
    ///
    /// Returns `true` if frame was allocated and is now freed.
    /// Returns `false` on out-of-range, unaligned address, or double-free.
    pub fn free_frame(&mut self, paddr: usize) -> bool {
        if paddr < self.base_paddr || !paddr.is_multiple_of(PAGE_SIZE) {
            return false;
        }
        let frame_idx = (paddr - self.base_paddr) / PAGE_SIZE;
        if frame_idx >= Self::TOTAL_FRAMES {
            return false;
        }

        let word_idx = frame_idx / 64;
        let bit_idx = frame_idx % 64;
        let mask = 1u64 << bit_idx;

        if (self.bitmap[word_idx] & mask) == 0 {
            return false; // double free
        }

        self.bitmap[word_idx] &= !mask;
        self.free_frames += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_free() {
        let mut alloc = FrameAllocator::<2>::new(0x8000_0000);
        assert_eq!(alloc.free_count(), 128);

        let f0 = alloc.alloc_frame().expect("frame 0");
        assert_eq!(f0, 0x8000_0000);
        assert_eq!(alloc.free_count(), 127);

        let f1 = alloc.alloc_frame().expect("frame 1");
        assert_eq!(f1, 0x8000_1000);

        assert!(alloc.free_frame(f0));
        assert_eq!(alloc.free_count(), 127);
        assert!(!alloc.free_frame(f0)); // double free rejected

        let f0_realloc = alloc.alloc_frame().expect("realloc frame 0");
        assert_eq!(f0_realloc, 0x8000_0000);
    }

    #[test]
    #[should_panic(expected = "base_paddr must be 4 KiB aligned")]
    fn test_unaligned_base_paddr_panics() {
        let _ = FrameAllocator::<1>::new(0x8000_0001);
    }

    #[test]
    fn test_alloc_frame_overflow_safety() {
        let mut alloc = FrameAllocator::<1>::new(usize::MAX - PAGE_SIZE + 1);
        assert!(alloc.alloc_frame().is_some());
        assert_eq!(alloc.alloc_frame(), None);
    }

    #[test]
    fn test_out_of_range_and_alignment() {
        let mut alloc = FrameAllocator::<1>::new(0x1000);
        assert!(!alloc.free_frame(0x800)); // below base
        assert!(!alloc.free_frame(0x1001)); // unaligned
        assert!(!alloc.free_frame(0x1000 + 64 * PAGE_SIZE)); // out of range
    }
}

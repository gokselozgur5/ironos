//! RISC-V Sv39 virtual memory paging structures and flags.

/// Sv39 Page Table Entry (64-bit).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    /// Valid bit.
    pub const V: u64 = 1 << 0;
    /// Readable bit.
    pub const R: u64 = 1 << 1;
    /// Writable bit.
    pub const W: u64 = 1 << 2;
    /// Executable bit.
    pub const X: u64 = 1 << 3;
    /// User mode accessible bit.
    pub const U: u64 = 1 << 4;
    /// Global mapping bit.
    pub const G: u64 = 1 << 5;
    /// Accessed bit.
    pub const A: u64 = 1 << 6;
    /// Dirty bit.
    pub const D: u64 = 1 << 7;

    /// Creates an empty (invalid) PTE.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Creates a PTE pointing to physical address `paddr` with permission `flags`.
    #[must_use]
    pub const fn new(paddr: usize, flags: u64) -> Self {
        let ppn = ((paddr >> 12) as u64) & 0x0FFF_FFFF_FFFF;
        Self((ppn << 10) | (flags & 0x3FF))
    }

    /// Returns the raw 64-bit value.
    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.0
    }

    /// Checks if PTE is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        (self.0 & Self::V) != 0
    }

    /// Checks if PTE is a leaf entry (readable, writable, or executable).
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        self.is_valid() && (self.0 & (Self::R | Self::W | Self::X)) != 0
    }

    /// Returns physical page number (PPN) stored in PTE.
    #[must_use]
    pub const fn ppn(&self) -> usize {
        ((self.0 >> 10) & 0x0FFF_FFFF_FFFF) as usize
    }

    /// Returns physical address pointed to by this PTE.
    #[must_use]
    pub const fn paddr(&self) -> usize {
        self.ppn() << 12
    }
}

/// Decoded Sv39 virtual address components.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sv39VirtAddr {
    /// `VPN[2]` index (bits 30..38, level 2 page table).
    pub vpn2: usize,
    /// `VPN[1]` index (bits 21..29, level 1 page table).
    pub vpn1: usize,
    /// `VPN[0]` index (bits 12..20, level 0 page table).
    pub vpn0: usize,
    /// Page offset (bits 0..11, 4 KiB offset).
    pub offset: usize,
}

impl Sv39VirtAddr {
    /// Decodes a 64-bit virtual address into Sv39 components.
    #[must_use]
    pub const fn from_vaddr(vaddr: usize) -> Self {
        Self {
            vpn2: (vaddr >> 30) & 0x1FF,
            vpn1: (vaddr >> 21) & 0x1FF,
            vpn0: (vaddr >> 12) & 0x1FF,
            offset: vaddr & 0xFFF,
        }
    }

    /// Checks if address is 4 KiB page-aligned.
    #[must_use]
    pub const fn is_aligned(&self) -> bool {
        self.offset == 0
    }

    /// Encodes components back into a 64-bit canonical virtual address.
    #[must_use]
    pub const fn to_vaddr(&self) -> usize {
        let v = ((self.vpn2 & 0x1FF) << 30)
            | ((self.vpn1 & 0x1FF) << 21)
            | ((self.vpn0 & 0x1FF) << 12)
            | (self.offset & 0xFFF);
        if (v & (1 << 38)) != 0 {
            v | (!0usize << 39)
        } else {
            v
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::PAGE_SIZE;

    #[test]
    fn test_sv39_vaddr_decode_encode() {
        let vaddr = 0x8020_1234;
        let decoded = Sv39VirtAddr::from_vaddr(vaddr);
        assert_eq!(decoded.offset, 0x234);
        assert!(!decoded.is_aligned());
        assert_eq!(decoded.to_vaddr(), vaddr);

        let aligned_vaddr = 0x8020_0000;
        let decoded_aligned = Sv39VirtAddr::from_vaddr(aligned_vaddr);
        assert_eq!(decoded_aligned.offset, 0);
        assert!(decoded_aligned.is_aligned());
        assert_eq!(decoded_aligned.to_vaddr(), aligned_vaddr);
    }

    #[test]
    fn test_sv39_vaddr_higher_half_canonical_roundtrip() {
        let higher_vaddr = 0xFFFF_FFC0_8020_0000usize;
        let decoded = Sv39VirtAddr::from_vaddr(higher_vaddr);
        assert_eq!(decoded.to_vaddr(), higher_vaddr);
    }

    #[test]
    fn test_page_table_entry_ppn_mask_and_invalid_leaf() {
        let raw_invalid = PageTableEntry::R | PageTableEntry::W;
        let pte_invalid = PageTableEntry(raw_invalid);
        assert!(!pte_invalid.is_valid());
        assert!(!pte_invalid.is_leaf());

        let paddr = 0x00FF_FFFF_FFFF_F000usize;
        let pte = PageTableEntry::new(paddr, PageTableEntry::V | PageTableEntry::R);
        assert!(pte.is_valid());
        assert!(pte.is_leaf());
        assert_eq!(pte.ppn(), 0x0FFF_FFFF_FFFFusize);
        assert_eq!(pte.paddr(), 0x00FF_FFFF_FFFF_F000usize);
    }

    #[test]
    fn test_page_table_entry() {
        let paddr = 0x8020_0000;
        let flags = PageTableEntry::V | PageTableEntry::R | PageTableEntry::X | PageTableEntry::A;
        let pte = PageTableEntry::new(paddr, flags);

        assert!(pte.is_valid());
        assert!(pte.is_leaf());
        assert_eq!(pte.paddr(), paddr);
        assert_eq!(pte.ppn(), paddr / PAGE_SIZE);

        let empty = PageTableEntry::empty();
        assert!(!empty.is_valid());
        assert!(!empty.is_leaf());
    }
}

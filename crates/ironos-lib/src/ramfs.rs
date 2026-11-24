//! In-memory fixed-capacity zero-alloc RAM filesystem.

/// Maximum length of a filename in bytes.
pub const MAX_NAME_LEN: usize = 24;

/// A single file stored in the in-memory RAM filesystem.
#[derive(Clone, Copy)]
pub struct RamFile<const MAX_DATA: usize> {
    name: [u8; MAX_NAME_LEN],
    name_len: usize,
    data: [u8; MAX_DATA],
    data_len: usize,
    used: bool,
}

impl<const MAX_DATA: usize> RamFile<MAX_DATA> {
    /// Creates an empty unused file slot.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            name: [0u8; MAX_NAME_LEN],
            name_len: 0,
            data: [0u8; MAX_DATA],
            data_len: 0,
            used: false,
        }
    }

    /// Returns the file name as a byte slice.
    #[must_use]
    pub fn name_bytes(&self) -> &[u8] {
        &self.name[..self.name_len]
    }

    /// Returns the file name as a string slice if valid UTF-8.
    #[must_use]
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(self.name_bytes()).unwrap_or("")
    }

    /// Returns the file contents.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_len]
    }
}

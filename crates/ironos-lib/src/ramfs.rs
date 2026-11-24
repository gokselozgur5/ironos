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

/// In-memory fixed-capacity zero-alloc RAM filesystem.
pub struct RamFs<const MAX_FILES: usize, const MAX_DATA: usize> {
    files: [RamFile<MAX_DATA>; MAX_FILES],
    count: usize,
}

impl<const MAX_FILES: usize, const MAX_DATA: usize> RamFs<MAX_FILES, MAX_DATA> {
    /// Creates an empty RAM filesystem instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            files: [RamFile::empty(); MAX_FILES],
            count: 0,
        }
    }

    /// Finds a file slot index by name if present.
    #[must_use]
    pub fn find_index(&self, name: &str) -> Option<usize> {
        let name_bytes = name.as_bytes();
        for (i, file) in self.files.iter().enumerate() {
            if file.used && file.name_bytes() == name_bytes {
                return Some(i);
            }
        }
        None
    }

    /// Reads file data by name if the file exists.
    #[must_use]
    pub fn read(&self, name: &str) -> Option<&[u8]> {
        let idx = self.find_index(name)?;
        Some(self.files[idx].data())
    }
}

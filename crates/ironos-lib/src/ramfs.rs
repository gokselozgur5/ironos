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

    /// Creates a file or updates its content if it already exists.
    pub fn create_or_write(&mut self, name: &str, data: &[u8]) -> Result<(), &'static str> {
        let name_bytes = name.as_bytes();
        if name_bytes.is_empty() || name_bytes.len() > MAX_NAME_LEN {
            return Err("invalid filename length");
        }
        if data.len() > MAX_DATA {
            return Err("file data exceeds maximum size");
        }

        if let Some(idx) = self.find_index(name) {
            let slot = &mut self.files[idx];
            slot.data[..data.len()].copy_from_slice(data);
            slot.data_len = data.len();
            return Ok(());
        }

        for slot in &mut self.files {
            if !slot.used {
                slot.name[..name_bytes.len()].copy_from_slice(name_bytes);
                slot.name_len = name_bytes.len();
                slot.data[..data.len()].copy_from_slice(data);
                slot.data_len = data.len();
                slot.used = true;
                self.count += 1;
                return Ok(());
            }
        }
        Err("ramfs file limit reached")
    }

    /// Returns the number of stored files.
    #[must_use]
    pub const fn file_count(&self) -> usize {
        self.count
    }

    /// Returns the total bytes occupied across all stored files.
    #[must_use]
    pub fn used_bytes(&self) -> usize {
        let mut total = 0;
        for file in &self.files {
            if file.used {
                total += file.data_len;
            }
        }
        total
    }

    /// Returns an iterator over all active files in the filesystem.
    pub fn files(&self) -> impl Iterator<Item = &RamFile<MAX_DATA>> {
        self.files.iter().filter(|f| f.used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramfs_create_read_overwrite() {
        let mut fs = RamFs::<4, 64>::new();
        assert_eq!(fs.file_count(), 0);
        assert_eq!(fs.used_bytes(), 0);

        fs.create_or_write("motd", b"ironos ready").unwrap();
        assert_eq!(fs.file_count(), 1);
        assert_eq!(fs.used_bytes(), 12);
        assert_eq!(fs.read("motd"), Some(&b"ironos ready"[..]));

        fs.create_or_write("motd", b"updated").unwrap();
        assert_eq!(fs.file_count(), 1);
        assert_eq!(fs.used_bytes(), 7);
        assert_eq!(fs.read("motd"), Some(&b"updated"[..]));
    }

    #[test]
    fn test_ramfs_limits_and_listing() {
        let mut fs = RamFs::<2, 16>::new();
        fs.create_or_write("f1", b"data1").unwrap();
        fs.create_or_write("f2", b"data2").unwrap();
        assert!(fs.create_or_write("f3", b"data3").is_err());
        assert_eq!(fs.file_count(), 2);
        assert!(fs.read("none").is_none());

        let mut count = 0;
        for file in fs.files() {
            count += 1;
            assert!(file.name_str() == "f1" || file.name_str() == "f2");
        }
        assert_eq!(count, 2);
    }
}

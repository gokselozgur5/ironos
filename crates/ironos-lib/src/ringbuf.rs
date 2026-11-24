//! Zero-alloc fixed-capacity byte ring buffer for UART and serial I/O.

/// Zero-allocation fixed-capacity byte ring buffer.
///
/// Capacity is `N` bytes.
#[derive(Debug)]
pub struct RingBuffer<const N: usize> {
    buf: [u8; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<const N: usize> RingBuffer<N> {
    /// Creates an empty ring buffer of capacity `N`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0u8; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    /// Returns capacity in bytes.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns current number of bytes stored.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if buffer is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if buffer is full.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Pushes a byte into the ring buffer.
    ///
    /// Returns `Ok(())` on success, or `Err(byte)` if the buffer is full.
    pub fn push(&mut self, byte: u8) -> Result<(), u8> {
        if self.is_full() {
            return Err(byte);
        }
        self.buf[self.tail] = byte;
        self.tail = (self.tail + 1) % N;
        self.len += 1;
        Ok(())
    }

    /// Pops a byte from the front of the ring buffer.
    pub fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        let byte = self.buf[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(byte)
    }
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop_and_wraparound() {
        let mut ring = RingBuffer::<4>::new();
        assert!(ring.is_empty());
        assert_eq!(ring.capacity(), 4);

        assert!(ring.push(b'A').is_ok());
        assert!(ring.push(b'B').is_ok());
        assert!(ring.push(b'C').is_ok());
        assert!(ring.push(b'D').is_ok());
        assert_eq!(ring.push(b'E'), Err(b'E'));
        assert!(ring.is_full());

        assert_eq!(ring.pop(), Some(b'A'));
        assert_eq!(ring.pop(), Some(b'B'));
        assert!(!ring.is_full());

        // Test wraparound
        assert!(ring.push(b'1').is_ok());
        assert!(ring.push(b'2').is_ok());
        assert!(ring.is_full());

        assert_eq!(ring.pop(), Some(b'C'));
        assert_eq!(ring.pop(), Some(b'D'));
        assert_eq!(ring.pop(), Some(b'1'));
        assert_eq!(ring.pop(), Some(b'2'));
        assert_eq!(ring.pop(), None);
        assert!(ring.is_empty());
    }
}

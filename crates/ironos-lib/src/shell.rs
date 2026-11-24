//! Interactive serial shell command parser and line editor.

/// Parsed interactive shell commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command<'a> {
    /// Show help text with list of commands.
    Help,
    /// Display memory and allocator usage statistics.
    Mem,
    /// Display active task table from scheduler.
    Ps,
    /// List files in the in-memory RAM filesystem.
    Ls,
    /// Display contents of a named file.
    Cat(&'a str),
    /// Create or overwrite file with given text content.
    Write(&'a str, &'a str),
    /// Display current timer tick counter.
    Ticks,
    /// Halt and shutdown the kernel.
    Halt,
    /// Empty line (no command entered).
    Empty,
    /// Unknown command string.
    Unknown(&'a str),
}

/// Parses an input line into a structured [`Command`].
#[must_use]
pub fn parse_command(line: &str) -> Command<'_> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Command::Empty;
    }

    let mut parts = trimmed.split_ascii_whitespace();
    let cmd = parts.next().unwrap_or("");
    match cmd {
        "help" => Command::Help,
        "mem" => Command::Mem,
        "ps" => Command::Ps,
        "ls" => Command::Ls,
        "cat" => {
            let name = parts.next().unwrap_or("");
            Command::Cat(name)
        }
        "write" => {
            let name = parts.next().unwrap_or("");
            let rest = trimmed.strip_prefix("write").unwrap_or("").trim_start();
            let content = rest.strip_prefix(name).unwrap_or("").trim_start();
            Command::Write(name, content)
        }
        "ticks" => Command::Ticks,
        "halt" => Command::Halt,
        _ => Command::Unknown(trimmed),
    }
}

/// Fixed-capacity line accumulator for interactive serial terminals.
pub struct LineBuffer<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> LineBuffer<N> {
    /// Creates an empty line editor buffer.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    /// Resets the buffer to empty.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Appends a byte or processes editing keys; returns `Some(&str)` when newline is submitted.
    pub fn push(&mut self, byte: u8) -> Option<&str> {
        match byte {
            b'\r' | b'\n' => {
                let s = core::str::from_utf8(&self.buf[..self.len]).unwrap_or("");
                Some(s)
            }
            0x08 | 0x7f => {
                if self.len > 0 {
                    self.len -= 1;
                }
                None
            }
            b if b >= 0x20 && b < 0x7f => {
                if self.len < N {
                    self.buf[self.len] = b;
                    self.len += 1;
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commands() {
        assert_eq!(parse_command("help"), Command::Help);
        assert_eq!(parse_command("  mem  "), Command::Mem);
        assert_eq!(parse_command("ps"), Command::Ps);
        assert_eq!(parse_command("ls"), Command::Ls);
        assert_eq!(parse_command("cat motd"), Command::Cat("motd"));
        assert_eq!(
            parse_command("write file hello world"),
            Command::Write("file", "hello world")
        );
        assert_eq!(parse_command("ticks"), Command::Ticks);
        assert_eq!(parse_command("halt"), Command::Halt);
        assert_eq!(parse_command("   "), Command::Empty);
        assert_eq!(parse_command("foo bar"), Command::Unknown("foo bar"));
    }

    #[test]
    fn test_line_buffer_edit() {
        let mut buf = LineBuffer::<16>::new();
        assert_eq!(buf.push(b'h'), None);
        assert_eq!(buf.push(b'i'), None);
        assert_eq!(buf.push(0x7f), None); // backspace
        assert_eq!(buf.push(b'a'), None);
        assert_eq!(buf.push(b'\n'), Some("ha"));
        buf.clear();
        assert_eq!(buf.push(b'\r'), Some(""));
    }
}

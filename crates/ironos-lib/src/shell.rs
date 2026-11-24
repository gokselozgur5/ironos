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

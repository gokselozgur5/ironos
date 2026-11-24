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

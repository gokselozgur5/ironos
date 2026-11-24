//! Task state representations for round-robin scheduler.

/// Maximum length of a task name in bytes.
pub const MAX_TASK_NAME_LEN: usize = 16;

/// State of a managed task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to be scheduled.
    Ready,
    /// Task is currently executing.
    Running,
    /// Task is blocked waiting for an event.
    Blocked,
    /// Task has completed execution.
    Exited,
}

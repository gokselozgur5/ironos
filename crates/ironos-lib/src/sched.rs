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

/// A single task control entry in the scheduler table.
#[derive(Clone, Copy)]
pub struct TaskEntry {
    /// Unique task identifier.
    pub id: usize,
    name: [u8; MAX_TASK_NAME_LEN],
    name_len: usize,
    /// Current task execution state.
    pub state: TaskState,
    /// Accumulated runtime ticks.
    pub ticks: u64,
    used: bool,
}

impl TaskEntry {
    /// Creates an empty task slot.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            id: 0,
            name: [0u8; MAX_TASK_NAME_LEN],
            name_len: 0,
            state: TaskState::Exited,
            ticks: 0,
            used: false,
        }
    }

    /// Returns the task name as a string slice.
    #[must_use]
    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    /// Returns whether this entry slot is in use.
    #[must_use]
    pub const fn is_used(&self) -> bool {
        self.used
    }
}

/// Fixed-slot round-robin cooperative/preemptive scheduler table.
pub struct Scheduler<const N: usize> {
    tasks: [TaskEntry; N],
    current: Option<usize>,
    next_id: usize,
}

impl<const N: usize> Scheduler<N> {
    /// Creates an empty scheduler table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tasks: [TaskEntry::empty(); N],
            current: None,
            next_id: 1,
        }
    }

    /// Spawns a new task in the first available slot.
    pub fn spawn(&mut self, name: &str) -> Result<usize, &'static str> {
        let name_bytes = name.as_bytes();
        if name_bytes.is_empty() || name_bytes.len() > MAX_TASK_NAME_LEN {
            return Err("invalid task name length");
        }
        for slot in &mut self.tasks {
            if !slot.used {
                slot.id = self.next_id;
                self.next_id += 1;
                slot.name[..name_bytes.len()].copy_from_slice(name_bytes);
                slot.name_len = name_bytes.len();
                slot.state = TaskState::Ready;
                slot.ticks = 0;
                slot.used = true;
                return Ok(slot.id);
            }
        }
        Err("scheduler table full")
    }
}

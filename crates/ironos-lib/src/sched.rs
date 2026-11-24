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

    /// Schedules the next ready task using round-robin policy.
    pub fn schedule_next(&mut self) -> Option<usize> {
        let start = match self.current {
            Some(idx) => (idx + 1) % N,
            None => 0,
        };

        for offset in 0..N {
            let idx = (start + offset) % N;
            if self.tasks[idx].used && self.tasks[idx].state == TaskState::Ready {
                if let Some(prev) = self.current {
                    if self.tasks[prev].used && self.tasks[prev].state == TaskState::Running {
                        self.tasks[prev].state = TaskState::Ready;
                    }
                }
                self.tasks[idx].state = TaskState::Running;
                self.current = Some(idx);
                return Some(self.tasks[idx].id);
            }
        }
        None
    }

    /// Increments accumulated tick count for the currently running task.
    pub fn on_tick(&mut self) {
        if let Some(idx) = self.current {
            if self.tasks[idx].used && self.tasks[idx].state == TaskState::Running {
                self.tasks[idx].ticks = self.tasks[idx].ticks.saturating_add(1);
            }
        }
    }

    /// Returns an iterator over all active tasks.
    pub fn tasks(&self) -> impl Iterator<Item = &TaskEntry> {
        self.tasks.iter().filter(|t| t.used)
    }
}

impl<const N: usize> Default for Scheduler<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_spawn_and_round_robin() {
        let mut sched = Scheduler::<4>::new();
        let id1 = sched.spawn("idle").unwrap();
        let id2 = sched.spawn("shell").unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        assert_eq!(sched.schedule_next(), Some(id1));
        sched.on_tick();
        sched.on_tick();

        assert_eq!(sched.schedule_next(), Some(id2));
        sched.on_tick();

        assert_eq!(sched.schedule_next(), Some(id1));

        let tasks: [TaskEntry; 4] = sched.tasks;
        assert_eq!(tasks[0].ticks, 2);
        assert_eq!(tasks[1].ticks, 1);
    }

    #[test]
    fn test_single_task_round_robin_keeps_running() {
        let mut sched = Scheduler::<4>::new();
        let id = sched.spawn("single").unwrap();
        assert_eq!(sched.schedule_next(), Some(id));
        sched.on_tick();
        assert_eq!(sched.schedule_next(), Some(id));
        sched.on_tick();
        assert_eq!(sched.schedule_next(), Some(id));
    }

    #[test]
    fn test_scheduler_limits() {
        let mut sched = Scheduler::<2>::new();
        assert!(sched.spawn("t1").is_ok());
        assert!(sched.spawn("t2").is_ok());
        assert!(sched.spawn("t3").is_err());
        assert!(sched.spawn("a_very_long_name_exceeding_limit").is_err());
    }
}

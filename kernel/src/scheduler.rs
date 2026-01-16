//! Task scheduler - manages which process runs when
//!
//! Implements a simple round-robin scheduler with ready queue.
//! The scheduler is responsible for:
//! - Maintaining ready task queue
//! - Selecting next task to run
//! - Handling context switches
//! - Supporting task suspension and resumption

use crate::process::ProcessStatus;
use alloc::collections::VecDeque;
use conquer_once::spin::OnceCell;
use spin::Mutex;

/// Scheduler state
pub struct Scheduler {
    /// Queue of ready processes waiting to run
    ready_queue: VecDeque<u64>,
    /// Current running process ID (None if idle)
    current_process: Option<u64>,
    /// Scheduling time quantum (timer ticks)
    time_quantum: usize,
    /// Current time counter
    time_counter: usize,
}

/// Global scheduler instance
static SCHEDULER: OnceCell<Mutex<Scheduler>> = OnceCell::uninit();

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Scheduler {
            ready_queue: VecDeque::new(),
            current_process: None,
            time_quantum: 100, // Default: 100 timer ticks per task
            time_counter: 0,
        }
    }

    /// Add a process to the ready queue
    pub fn enqueue(&mut self, pid: u64) {
        if !self.ready_queue.contains(&pid) {
            self.ready_queue.push_back(pid);
        }
    }

    /// Remove a process from the ready queue
    pub fn dequeue(&mut self) -> Option<u64> {
        self.ready_queue.pop_front()
    }

    /// Get the current running process
    pub fn current(&self) -> Option<u64> {
        self.current_process
    }

    /// Set the current running process
    pub fn set_current(&mut self, pid: Option<u64>) {
        self.current_process = pid;
    }

    /// Increment time counter and check if time quantum expired
    pub fn tick(&mut self) -> bool {
        self.time_counter += 1;
        if self.time_counter >= self.time_quantum {
            self.time_counter = 0;
            true // Time quantum expired, need to context switch
        } else {
            false
        }
    }

    /// Select next process to run (round-robin)
    /// Returns (previous_pid, next_pid)
    pub fn schedule(&mut self) -> (Option<u64>, Option<u64>) {
        let prev = self.current_process;

        // Put current process back in queue if not blocked/exited
        if let Some(pid) = self.current_process {
            if let Some(status) = crate::process::get_process_status(pid) {
                match status {
                    ProcessStatus::Running => {
                        // Process was running, move to ready queue
                        self.enqueue(pid);
                    }
                    ProcessStatus::Blocked | ProcessStatus::Exited(_) => {
                        // Don't re-queue blocked or exited processes
                    }
                    _ => {}
                }
            }
        }

        // Get next process from ready queue
        let next = self.dequeue();
        self.current_process = next;

        (prev, next)
    }
}

/// Get or initialize the scheduler
fn get_or_init_scheduler() -> &'static Mutex<Scheduler> {
    SCHEDULER.get_or_init(|| Mutex::new(Scheduler::new()))
}

/// Add a process to the scheduler ready queue
pub fn enqueue_process(pid: u64) {
    let scheduler = get_or_init_scheduler();
    let mut sched = scheduler.lock();
    sched.enqueue(pid);
}

/// Get the current running process
pub fn current_process() -> Option<u64> {
    let scheduler = get_or_init_scheduler();
    let sched = scheduler.lock();
    sched.current()
}

/// Timer interrupt handler - call on each timer tick
/// Returns true if context switch is needed
pub fn timer_tick() -> bool {
    let scheduler = get_or_init_scheduler();
    let mut sched = scheduler.lock();
    sched.tick()
}

/// Perform round-robin scheduling
/// Returns (current_pid_to_save, next_pid_to_load)
pub fn schedule() -> (Option<u64>, Option<u64>) {
    let scheduler = get_or_init_scheduler();
    let mut sched = scheduler.lock();
    sched.schedule()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_enqueue() {
        let mut sched = Scheduler::new();
        sched.enqueue(1);
        sched.enqueue(2);
        assert_eq!(sched.dequeue(), Some(1));
        assert_eq!(sched.dequeue(), Some(2));
    }

    #[test]
    fn test_scheduler_round_robin() {
        let mut sched = Scheduler::new();
        sched.enqueue(1);
        sched.enqueue(2);
        sched.enqueue(3);

        // After 3 dequeues, should be empty
        sched.dequeue();
        sched.dequeue();
        sched.dequeue();
        assert_eq!(sched.dequeue(), None);
    }

    #[test]
    fn test_time_quantum() {
        let mut sched = Scheduler::new();
        for _ in 0..99 {
            assert_eq!(sched.tick(), false);
        }
        assert_eq!(sched.tick(), true); // Should expire after 100 ticks
    }
}

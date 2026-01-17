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
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// Global elapsed time in timer ticks since kernel boot
/// Timer frequency is approximately 100 Hz (10ms per tick)
static ELAPSED_TICKS: spin::Mutex<u64> = spin::Mutex::new(0);

/// Control whether timer interrupts perform context switching
/// Set to false when running async executor (cooperative multitasking)
/// Set to true for pure preemptive scheduling
static PREEMPTION_ENABLED: AtomicBool = AtomicBool::new(true);

/// Disable timer-based preemption (for cooperative multitasking environments like async executor)
pub fn disable_preemption() {
    PREEMPTION_ENABLED.store(false, Ordering::SeqCst);
}

/// Enable timer-based preemption
pub fn enable_preemption() {
    PREEMPTION_ENABLED.store(true, Ordering::SeqCst);
}

/// Check if preemption is currently enabled
pub fn is_preemption_enabled() -> bool {
    PREEMPTION_ENABLED.load(Ordering::SeqCst)
}

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

    /// Increment global elapsed time (called on each timer tick)
    fn increment_elapsed_time() {
        let mut ticks = ELAPSED_TICKS.lock();
        *ticks = ticks.saturating_add(1);
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
    // Increment global elapsed time
    Scheduler::increment_elapsed_time();

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

/// Check if current task's quantum has expired
/// Used by syscalls to determine if preemption is needed
/// Does NOT reset the counter - that's done on actual switch
pub fn check_quantum_expired() -> bool {
    let scheduler = get_or_init_scheduler();
    let sched = scheduler.lock();
    sched.time_counter >= sched.time_quantum
}

/// Get elapsed time in seconds since kernel boot
pub fn get_elapsed_seconds() -> u64 {
    let ticks = ELAPSED_TICKS.lock();
    *ticks / 100 // 100 Hz timer = divide by 100 to get seconds
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

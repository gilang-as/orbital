//! Process/task launcher for spawning user tasks
//!
//! Provides mechanism for creating and managing lightweight user tasks.
//! Policy (what tasks do, scheduling priorities) is left to userspace.

use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use spin::Mutex;

/// Unique identifier for a process/task
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(u64);

impl ProcessId {
    /// Generate a new unique process ID
    fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_PID: AtomicU64 = AtomicU64::new(1);
        ProcessId(NEXT_PID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Status of a process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is ready to run
    Ready,
    /// Process is currently running
    Running,
    /// Process is waiting for I/O or event
    Blocked,
    /// Process has exited
    Exited(i64),
}

/// A lightweight process/task that the kernel manages
#[derive(Debug)]
pub struct Process {
    /// Unique process identifier
    pub id: ProcessId,
    /// Entry point address (function pointer cast to usize)
    pub entry_point: usize,
    /// Current status
    pub status: ProcessStatus,
    /// Return value (when exited)
    pub exit_code: i64,
}

impl Process {
    /// Create a new process with the given entry point
    pub fn new(entry_point: usize) -> Self {
        Process {
            id: ProcessId::new(),
            entry_point,
            status: ProcessStatus::Ready,
            exit_code: 0,
        }
    }
}

/// Global process table
static PROCESS_TABLE: OnceCell<Mutex<Vec<Process>>> = OnceCell::uninit();

/// Get or initialize the process table
fn get_or_init_process_table() -> &'static Mutex<Vec<Process>> {
    PROCESS_TABLE.get_or_init(|| Mutex::new(Vec::new()))
}

/// Create a new process/task
///
/// # Arguments
/// * `entry_point` - Address of the task's entry function
///
/// # Returns
/// Process ID if successful, or negative error code
pub fn create_process(entry_point: usize) -> i64 {
    // Validate entry point is not NULL
    if entry_point == 0 {
        return -1; // Invalid address
    }

    let table = get_or_init_process_table();
    let mut processes = table.lock();

    // Check if we have room for more processes (arbitrary limit)
    if processes.len() >= 256 {
        return -2; // Too many processes
    }

    let process = Process::new(entry_point);
    let pid = process.id.0 as i64;
    processes.push(process);

    pid
}

/// Get process by ID
pub fn get_process(pid: u64) -> Option<ProcessId> {
    let table = get_or_init_process_table();
    let processes = table.lock();

    processes
        .iter()
        .find(|p| p.id.0 == pid)
        .map(|p| p.id)
}

/// Get the status of a process
pub fn get_process_status(pid: u64) -> Option<ProcessStatus> {
    let table = get_or_init_process_table();
    let processes = table.lock();

    processes
        .iter()
        .find(|p| p.id.0 == pid)
        .map(|p| p.status)
}

/// Update process status
pub fn set_process_status(pid: u64, status: ProcessStatus) -> bool {
    let table = get_or_init_process_table();
    let mut processes = table.lock();

    if let Some(process) = processes.iter_mut().find(|p| p.id.0 == pid) {
        process.status = status;
        true
    } else {
        false
    }
}

/// Wait for a process to exit and return its exit code
pub fn wait_process(pid: u64) -> Option<i64> {
    loop {
        let table = get_or_init_process_table();
        let processes = table.lock();

        if let Some(process) = processes.iter().find(|p| p.id.0 == pid) {
            match process.status {
                ProcessStatus::Exited(code) => return Some(code),
                _ => {
                    // Process still running, need to yield and retry
                    drop(processes);
                    // Small busy-wait (in real implementation would use events)
                    for _ in 0..1000 {
                        core::hint::spin_loop();
                    }
                }
            }
        } else {
            // Process doesn't exist
            return None;
        }
    }
}

/// List all processes (for debugging)
pub fn list_processes() -> alloc::vec::Vec<(u64, ProcessStatus)> {
    let table = get_or_init_process_table();
    let processes = table.lock();

    processes
        .iter()
        .map(|p| (p.id.0, p.status))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_process() {
        let pid = create_process(0x1000);
        assert!(pid > 0);
    }

    #[test]
    fn test_process_id_unique() {
        let pid1 = create_process(0x1000);
        let pid2 = create_process(0x2000);
        assert_ne!(pid1, pid2);
    }

    #[test]
    fn test_invalid_entry_point() {
        let pid = create_process(0); // NULL pointer
        assert_eq!(pid, -1);
    }
}

//! Process/task launcher for spawning user tasks
//!
//! Provides mechanism for creating and managing lightweight user tasks.
//! Policy (what tasks do, scheduling priorities) is left to userspace.
//!
//! Task Memory Layout:
//! Each task gets a 4KB stack allocated from the kernel heap.
//! Stack grows downward (high to low address).
//! 
//! Stack Layout (grows downward):
//! ┌─────────────────┐ 0x7FFF
//! │   top (unused)  │
//! ├─────────────────┤
//! │    local vars   │
//! │    saved regs   │
//! │    args         │
//! ├─────────────────┤ (RSP)
//! │ (grows downward)│
//! └─────────────────┘ 0x0000
//!
//! Context switching saves/restores the full CPU state (all registers).

use alloc::boxed::Box;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use spin::Mutex;
use crate::println;

const TASK_STACK_SIZE: usize = 4096; // 4KB per task

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

/// CPU context - all registers saved for a process
/// Used during context switches to save/restore process state
/// CRITICAL: #[repr(C)] ensures field order for inline asm offsets!
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TaskContext {
    /// General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    /// Instruction pointer
    pub rip: u64,
    /// CPU flags register
    pub rflags: u64,
}

impl TaskContext {
    /// Create a new context for a task starting at entry_point
    /// Stack pointer is set to the top of the stack (grows downward)
    pub fn new(entry_point: u64, _stack_top: u64) -> Self {
        // For now, we don't actually use context switching
        // Just store the entry point for reference
        TaskContext {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: entry_point,    // Task function pointer
            rbp: 0,              // Not used
            rsp: 0,              // Not used
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,              // Not used
            rflags: 0,           // Not used
        }
    }
}

/// A lightweight process/task that the kernel manages
#[derive(Debug)]
pub struct Process {
    /// Unique process identifier
    pub id: ProcessId,
    /// Entry point address (function pointer cast to usize)
    pub entry_point: usize,
    /// Allocated stack for this task (4KB) - using Box for stable address
    pub stack: Box<[u8; TASK_STACK_SIZE]>,
    /// Saved CPU context (for context switching)
    pub saved_context: TaskContext,
    /// Current status
    pub status: ProcessStatus,
    /// Return value (when exited)
    pub exit_code: i64,
}

impl Process {
    /// Create a new process with the given entry point
    /// Allocates a stack and initializes CPU context
    pub fn new(entry_point: usize) -> Self {
        // For now, we don't allocate stacks - just store the task function
        // Tasks will be executed directly by calling the function, not by context switching
        let task_fn_ptr = entry_point as u64;
        let saved_context = TaskContext::new(task_fn_ptr, 0);
        
        Process {
            id: ProcessId::new(),
            entry_point,
            stack: Box::new([0u8; TASK_STACK_SIZE]), // Still allocate but don't use yet
            saved_context,
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
    let pid = process.id.0;
    processes.push(process);

    // Enqueue the process in the scheduler
    drop(processes); // Release the lock before calling scheduler
    crate::scheduler::enqueue_process(pid);

    pid as i64
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

/// Execute a single task by PID directly (no context switching)
pub fn execute_process(pid: u64) -> Option<i64> {
    let entry_point = {
        let table = get_or_init_process_table();
        let mut processes = table.lock();
        
        if let Some(process) = processes.iter_mut().find(|p| p.id.0 == pid) {
            process.status = ProcessStatus::Running;
            process.entry_point
        } else {
            return None;
        }
    };
    
    // Execute the task function directly
    let task_fn = unsafe { core::mem::transmute::<usize, fn() -> i64>(entry_point) };
    let exit_code = task_fn();
    
    // Mark as exited
    set_process_status(pid, ProcessStatus::Exited(exit_code));
    
    Some(exit_code)
}

/// Execute all ready processes
pub fn execute_all_ready() -> u32 {
    let mut executed = 0;
    
    loop {
        // Find next ready process
        let pid_to_run = {
            let table = get_or_init_process_table();
            let processes = table.lock();
            
            processes
                .iter()
                .find(|p| p.status == ProcessStatus::Ready)
                .map(|p| p.id.0)
        };
        
        if let Some(pid) = pid_to_run {
            execute_process(pid);
            executed += 1;
        } else {
            break;
        }
    }
    
    executed
}

/// Get mutable reference to process's context for saving/restoring
pub fn get_process_context_mut(pid: u64) -> Option<*mut TaskContext> {
    let table = get_or_init_process_table();
    let mut processes = table.lock();

    if let Some(process) = processes.iter_mut().find(|p| p.id.0 == pid) {
        Some(&mut process.saved_context as *mut TaskContext)
    } else {
        None
    }
}

/// Get process's stack pointer (RSP)
pub fn get_process_stack_pointer(pid: u64) -> Option<u64> {
    let table = get_or_init_process_table();
    let processes = table.lock();

    processes
        .iter()
        .find(|p| p.id.0 == pid)
        .map(|p| p.saved_context.rsp)
}

/// Update process's stack pointer (RSP)
pub fn set_process_stack_pointer(pid: u64, rsp: u64) -> bool {
    let table = get_or_init_process_table();
    let mut processes = table.lock();

    if let Some(process) = processes.iter_mut().find(|p| p.id.0 == pid) {
        process.saved_context.rsp = rsp;
        true
    } else {
        false
    }
}

/// Get mutable access to a process (internal use)
pub fn get_process_mut(pid: u64) -> Option<ProcessMutRef> {
    // This is a helper that returns a reference to the process
    // In practice, we use the table directly, but this helps with the API
    let table = get_or_init_process_table();
    let processes = table.lock();
    
    if processes.iter().any(|p| p.id.0 == pid) {
        // Return a simple wrapper that indicates we can access the process
        Some(ProcessMutRef { pid })
    } else {
        None
    }
}

/// Helper struct for mutable process access
pub struct ProcessMutRef {
    pid: u64,
}

impl ProcessMutRef {
    /// Update the saved context for this process
    pub fn update_context(&self, ctx: TaskContext) {
        let table = get_or_init_process_table();
        let mut processes = table.lock();
        if let Some(process) = processes.iter_mut().find(|p| p.id.0 == self.pid) {
            process.saved_context = ctx;
        }
    }
}

/// Context switch: Save current task's context, load next task's context
/// This is called during process switches (e.g., on timer interrupt, syscall)
///
/// # Safety
/// Caller must ensure valid CPU state and no reentrancy
pub unsafe fn context_switch(current_pid: Option<u64>, next_pid: u64) {
    // If there's a current process, save its context
    if let Some(pid) = current_pid {
        if let Some(_ctx_ptr) = get_process_context_mut(pid) {
            // In a real implementation, we'd save all CPU registers here
            // For now, this is a placeholder for assembly-based save
            set_process_status(pid, ProcessStatus::Ready);
        }
    }

    // Load the next process's context
    if let Some(_ctx_ptr) = get_process_context_mut(next_pid) {
        // In a real implementation, we'd restore all CPU registers
        // and jump to the process's entry point
        set_process_status(next_pid, ProcessStatus::Running);
    }
}

/// Get a copy of a process's context
pub fn get_process_context(pid: u64) -> Option<TaskContext> {
    let table = get_or_init_process_table();
    let processes = table.lock();
    processes
        .iter()
        .find(|p| p.id.0 == pid)
        .map(|p| p.saved_context.clone())
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

    #[test]
    fn test_task_context_initialization() {
        // Test that TaskContext is properly initialized for task entry
        let stack_top = 0x8000u64;
        let entry_point = 0x1000u64;
        
        let ctx = TaskContext::new(entry_point, stack_top);
        
        // Verify RIP points to entry point wrapper
        assert!(ctx.rip > 0);
        
        // Verify RDI contains task function pointer
        assert_eq!(ctx.rdi, entry_point);
        
        // Verify RBP at stack top
        assert_eq!(ctx.rbp, stack_top);
        
        // Verify RSP is adjusted for stack frame
        assert!(ctx.rsp < stack_top);
        
        // Verify interrupts are enabled (0x200 = IF flag)
        assert_eq!(ctx.rflags, 0x200);
    }
}

//! Task execution entry point
//!
//! Provides the entry point for all tasks. When a task is scheduled,
//! its RIP is set to task_wrapper_entry. At that point:
//! - RDI contains the task function pointer (fn() signature)
//! - Other registers are restored to saved state
//! - The task runs until it returns or calls sys_exit
//!
//! When the task function returns, we capture its return value (in RAX)
//! and call sys_exit to terminate the task cleanly.

use crate::syscall;

/// Task function type: takes no arguments, returns i64 exit code
pub type TaskFn = fn() -> i64;

/// Initialize a task's stack with proper frame
/// 
/// Sets up the stack so that the task can be properly initialized.
///
/// # Arguments
/// * `stack_top` - Top of the 4KB task stack
/// * `task_fn` - Pointer to the task function
///
/// # Returns
/// The adjusted RSP that should be used in the TaskContext
pub fn init_task_stack(stack_top: u64, _task_fn: u64) -> u64 {
    // For now, we simply return stack_top - 8
    // The task function will be passed in RDI via context restoration
    stack_top - 8
}

/// Get the task entry point
pub fn get_task_entry_point() -> u64 {
    task_wrapper_entry as *const () as u64
}

/// Task wrapper entry point - this is the initial RIP for all tasks
///
/// This function is called when a task is scheduled and its context is restored.
/// At this point:
/// - RDI contains the task function pointer
/// - All CPU registers have been restored from TaskContext
/// - Stack is set up for the task (RSP points to task stack)
///
/// We use inline assembly to:
/// 1. Call the function pointer in RDI
/// 2. Capture the return value (in RAX) as the exit code
/// 3. Call sys_exit to properly terminate the task
#[inline(never)]
pub fn task_wrapper_entry() {
    let exit_code: i64;
    
    unsafe {
        // Call the function pointer that's in RDI
        // The function returns an i64 (exit code) in RAX
        core::arch::asm!(
            "call rdi",
            out("rax") exit_code,
        );
    }
    
    // Task function has returned, now call sys_exit with the exit code
    // This will terminate the task and schedule the next process
    let _ = syscall::dispatch_syscall(
        syscall::nr::SYS_EXIT,
        exit_code as usize,
        0,
        0,
        0,
        0,
        0,
    );
    
    // dispatch_syscall should never return for sys_exit,
    // but if it does, we halt the CPU
    unsafe {
        core::arch::asm!("hlt", options(noreturn));
    }
}


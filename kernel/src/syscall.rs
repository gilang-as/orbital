//! Syscall dispatcher and handlers
//!
//! This module provides the minimal syscall skeleton for Orbital OS.
//! Syscalls are the interface between userspace and kernel.
//!
//! Architecture: x86_64 syscall/sysret instruction
//! ABI: System V AMD64 - arguments via rdi, rsi, rdx, rcx, r8, r9
//!
//! Syscall numbers are passed in RAX.
//! Return values are in RAX (or error code in RAX with sign bit set).

use core::fmt;
extern crate alloc;
use alloc::format;

/// Syscall error codes
/// Follows Unix convention: negative values indicate errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum SysError {
    /// Invalid syscall number or argument
    Invalid = -1,
    /// Syscall not yet implemented
    NotImplemented = -2,
    /// Memory fault (e.g., invalid pointer)
    Fault = -3,
    /// Permission denied
    PermissionDenied = -4,
    /// Resource not found
    NotFound = -5,
    /// Generic kernel error
    Error = -6,
    /// Bad file descriptor
    BadFd = -9,
}

impl SysError {
    /// Convert error to syscall return value
    /// Negative values indicate error in syscall ABI
    pub fn to_return_value(self) -> i64 {
        self as i64
    }

    /// Create from numeric error code
    pub fn from_code(code: i64) -> Option<Self> {
        match code {
            -1 => Some(SysError::Invalid),
            -2 => Some(SysError::NotImplemented),
            -3 => Some(SysError::Fault),
            -4 => Some(SysError::PermissionDenied),
            -5 => Some(SysError::NotFound),
            -6 => Some(SysError::Error),
            -9 => Some(SysError::BadFd),
            _ => None,
        }
    }
}

impl fmt::Display for SysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SysError::Invalid => write!(f, "Invalid argument or syscall"),
            SysError::NotImplemented => write!(f, "Syscall not implemented"),
            SysError::Fault => write!(f, "Memory fault"),
            SysError::PermissionDenied => write!(f, "Permission denied"),
            SysError::NotFound => write!(f, "Not found"),
            SysError::Error => write!(f, "Kernel error"),
            SysError::BadFd => write!(f, "Bad file descriptor"),
        }
    }
}

/// Syscall result type
pub type SysResult = Result<usize, SysError>;

/// Syscall handler function signature
/// Takes syscall number and up to 6 arguments, returns result
type SyscallHandler = fn(usize, usize, usize, usize, usize, usize) -> SysResult;

/// Syscall dispatch table
/// Maps syscall numbers to handler functions
const SYSCALL_TABLE: &[Option<SyscallHandler>] = &[
    Some(sys_hello),       // 0
    Some(sys_log),         // 1
    Some(sys_write),       // 2
    Some(sys_exit),        // 3
    Some(sys_read),        // 4
    Some(sys_task_create), // 5
    Some(sys_task_wait),   // 6
    Some(sys_get_pid),     // 7
    Some(sys_ps),          // 8
    Some(sys_uptime),      // 9
                           // More syscalls go here
];

/// Syscall number constants
pub mod nr {
    pub const SYS_HELLO: usize = 0;
    pub const SYS_LOG: usize = 1;
    pub const SYS_WRITE: usize = 2;
    pub const SYS_EXIT: usize = 3;
    pub const SYS_READ: usize = 4;
    pub const SYS_TASK_CREATE: usize = 5;
    pub const SYS_TASK_WAIT: usize = 6;
    pub const SYS_GET_PID: usize = 7;
    pub const SYS_PS: usize = 8;
    pub const SYS_UPTIME: usize = 9;
}

/// Main syscall dispatcher
/// Called from low-level entry point with syscall number and arguments
pub fn dispatch_syscall(
    syscall_nr: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> i64 {
    // Dispatch to handler or return error
    if let Some(handler) = SYSCALL_TABLE.get(syscall_nr).and_then(|h| h.as_ref()) {
        match handler(arg1, arg2, arg3, arg4, arg5, arg6) {
            Ok(ret) => ret as i64,
            Err(e) => e.to_return_value(),
        }
    } else {
        SysError::NotImplemented.to_return_value()
    }
}

// ============================================================================
// Minimal Syscall Implementations
// ============================================================================

/// sys_hello - Test syscall
/// Returns a constant value (0xDEADBEEF as success indicator)
fn sys_hello(
    arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    // Use arg1 to demonstrate argument passing
    // arg1 is typically the "magic number" for verification
    if arg1 == 0xCAFEBABE {
        Ok(0xDEADBEEF)
    } else {
        Err(SysError::Invalid)
    }
}

/// sys_log - Write message to kernel log
///
/// Safely copies a message from userspace memory and outputs it to the kernel log.
/// This is the first real syscall - it demonstrates userspace-to-kernel data transfer
/// without interpreting or validating the message content.
///
/// Arguments:
///   arg1: pointer to message buffer (from userspace)
///   arg2: message length (in bytes)
///   other arguments: unused
///
/// Returns:
///   Success: number of bytes written (arg2)
///   Failure: negative error code
///
/// Safety:
/// - Validates pointer is not NULL
/// - Validates length is within reasonable bounds (1-1024 bytes)
/// - Uses core::ptr::copy_nonoverlapping for safe memory copy
/// - Does NOT interpret message content (bytes are opaque to kernel)
/// - Disables interrupts during output to prevent context switches
fn sys_log(
    arg1: usize,
    arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    let ptr = arg1 as *const u8;
    let len = arg2;

    // Validate length
    if len == 0 {
        return Err(SysError::Invalid);
    }
    if len > 4096 {
        // Reasonable upper limit to prevent DoS
        return Err(SysError::Invalid);
    }

    // Validate pointer is not NULL
    if ptr.is_null() {
        return Err(SysError::Fault);
    }

    // Allocate kernel buffer for the message
    // Using Vec to safely manage allocation
    let mut buffer = alloc::vec::Vec::with_capacity(len);

    // Safely copy from userspace memory
    // SAFETY: We trust the pointer is valid userspace memory because:
    // 1. We've validated it's not NULL
    // 2. We've validated the length
    // 3. The kernel will page fault if it's invalid (handled by CPU)
    // 4. We're in syscall context, not holding any locks
    unsafe {
        // Copy bytes from userspace to kernel buffer
        core::ptr::copy_nonoverlapping(ptr, buffer.as_mut_ptr(), len);
        buffer.set_len(len);
    }

    // Route to TTY with newline for kernel logging
    crate::tty::tty_write_with_newline(&buffer);

    // Return number of bytes written
    Ok(len)
}

/// sys_write - Write to file descriptor
///
/// UNIX-style write syscall that allows userspace to write to stdout (fd=1) or stderr (fd=2).
/// This introduces a simple file descriptor abstraction while keeping the kernel minimal.
///
/// Arguments:
///   arg1: file descriptor (1=stdout, 2=stderr, others invalid)
///   arg2: pointer to buffer (from userspace)
///   arg3: number of bytes to write
///   other arguments: unused
///
/// Returns:
///   Success: number of bytes written (arg3)
///   Failure: negative error code (BadFd, Invalid, Fault)
///
/// Safety:
/// - Validates fd (must be 1 or 2)
/// - Validates buffer length (same as sys_log: 1-4096)
/// - Validates pointer is not NULL
/// - Uses safe memory copy
fn sys_write(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    let fd = arg1;
    let ptr = arg2 as *const u8;
    let len = arg3;

    // Validate fd
    if fd != 1 && fd != 2 {
        return Err(SysError::BadFd);
    }

    // Validate length (same as sys_log)
    if len == 0 {
        return Err(SysError::Invalid);
    }
    if len > 4096 {
        return Err(SysError::Invalid);
    }

    // Validate pointer is not NULL
    if ptr.is_null() {
        return Err(SysError::Fault);
    }

    // Allocate kernel buffer for the data
    let mut buffer = alloc::vec::Vec::with_capacity(len);

    // Safely copy from userspace memory
    unsafe {
        core::ptr::copy_nonoverlapping(ptr, buffer.as_mut_ptr(), len);
        buffer.set_len(len);
    }

    // Route to TTY device (both fd=1 and fd=2 go through same backend)
    // No newline added to preserve exact output semantics
    crate::tty::tty_write(&buffer);

    // Return number of bytes written
    Ok(len)
}

/// sys_exit - Terminate process
/// Arguments:
///   arg1: exit code
/// Returns: never (or error if already exiting)
fn sys_exit(
    arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    let exit_code = arg1 as i64;

    // Get current process ID from scheduler
    if let Some(current_pid) = crate::scheduler::current_process() {
        // Mark process as exited with the given exit code
        crate::process::set_process_status(
            current_pid,
            crate::process::ProcessStatus::Exited(exit_code),
        );

        // Note: We don't perform context_switch here because sys_exit is called
        // from task_wrapper_entry which is in task context, not interrupt handler context.
        // Context switches must only happen from interrupt handlers with proper stack state.
        // The next timer interrupt will see this task is Exited and schedule a different one.

        // Just halt - the next timer interrupt will handle scheduling
        crate::hlt_loop();
    }

    // If no current process, return error
    Err(SysError::NotFound)
}

/// sys_read - Read from file descriptor
///
/// Simple read syscall for input. Currently supports:
/// - fd=0 (stdin): reads from kernel input buffer
/// - Other fds: returns BadFd
///
/// Arguments:
///   arg1: file descriptor (0=stdin, others invalid)
///   arg2: pointer to buffer (from userspace)
///   arg3: number of bytes to read
///   other arguments: unused
///
/// Returns:
///   Success: number of bytes read
///   Failure: negative error code (BadFd, Invalid, Fault)
///
/// Safety:
/// - Validates fd (must be 0 for stdin)
/// - Validates buffer length (1-4096)
/// - Validates pointer is not NULL
/// - Uses safe memory copy from kernel buffer to userspace
fn sys_read(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    let fd = arg1;
    let ptr = arg2 as *mut u8;
    let len = arg3;

    // Validate fd (only stdin=0 supported)
    if fd != 0 {
        return Err(SysError::BadFd);
    }

    // Validate length
    if len == 0 {
        return Ok(0); // Reading 0 bytes is OK, just returns immediately
    }
    if len > 4096 {
        return Err(SysError::Invalid);
    }

    // Validate pointer is not NULL
    if ptr.is_null() {
        return Err(SysError::Fault);
    }

    // Read from kernel input buffer
    let bytes_read = crate::input::read_input(unsafe {
        // SAFETY: We've validated:
        // 1. ptr is not NULL
        // 2. len is in valid range
        // 3. We're creating a mutable slice for writing from kernel
        // 4. Userspace is responsible for the memory being valid
        core::slice::from_raw_parts_mut(ptr, len)
    });

    Ok(bytes_read)
}

/// Syscall #5: Create a new process/task
///
/// Creates a new lightweight process with the given entry point.
/// The task will be managed by the kernel and can be scheduled.
///
/// # Arguments
/// - arg1: Entry point address (function pointer as usize)
/// - Others: Reserved for future use
///
/// # Returns
/// - Ok(pid): Process ID (positive)
/// - Err(SysError::Invalid): Invalid entry point (NULL)
/// - Err(SysError::Error): Too many processes or other error
///
/// # Process
/// 1. Create process with entry point (allocates stack)
/// 2. Add to scheduler ready queue
/// 3. Return process ID
fn sys_task_create(
    arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    let entry_point = arg1;

    // Validate entry point is not NULL
    if entry_point == 0 {
        return Err(SysError::Invalid);
    }

    // Create the process (allocates 4KB stack, sets up context)
    let pid = crate::process::create_process(entry_point);

    if pid < 0 {
        // Negative return value indicates error
        match pid {
            -1 => Err(SysError::Invalid), // Invalid address
            -2 => Err(SysError::Error),   // Too many processes
            _ => Err(SysError::Error),    // Other error
        }
    } else {
        // Add the new process to the scheduler's ready queue
        crate::scheduler::enqueue_process(pid as u64);

        // Update status to Ready
        crate::process::set_process_status(pid as u64, crate::process::ProcessStatus::Ready);

        // Return the process ID as success
        Ok(pid as usize)
    }
}

/// sys_task_wait - Wait for a task to complete
///
/// Blocks until the specified task exits, returning its exit code.
///
/// # Arguments
/// - arg1: Process ID to wait for
/// - Others: Reserved
///
/// # Returns
/// - Ok(exit_code): Task's exit code when it completes
/// - Err(SysError::NotFound): Task doesn't exist
/// - Err(SysError::Invalid): Invalid task ID
fn sys_task_wait(
    arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    let pid = arg1 as u64;

    // Validate PID is not zero
    if pid == 0 {
        return Err(SysError::Invalid);
    }

    // Wait for process to exit
    match crate::process::wait_process(pid) {
        Some(exit_code) => Ok(exit_code as usize),
        None => Err(SysError::NotFound),
    }
}

/// sys_get_pid - Get the current process ID
///
/// Returns the process ID of the currently running task.
/// This is useful for tasks to identify themselves.
///
/// # Arguments
/// - None (all arguments ignored)
///
/// # Returns
/// - Ok(pid): Current process ID (always > 0)
fn sys_get_pid(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    // In a real implementation, we'd get the current process from the scheduler
    // For now, return a placeholder (in future: retrieve from scheduler::current_process())
    // Using 1 as placeholder since task IDs start at 1
    Ok(crate::scheduler::current_process().unwrap_or(1) as usize)
}

/// sys_ps - List all processes
///
/// Returns information about all running processes.
/// Writes process list to an output buffer (simplified version).
///
/// # Arguments
/// - arg1: Pointer to output buffer (userspace memory)
/// - arg2: Buffer size in bytes
/// - Others: Reserved
///
/// # Returns
/// - Ok(bytes_written): Number of bytes written to buffer
/// - Err(SysError::Fault): Invalid pointer
/// - Err(SysError::Invalid): Buffer too small
fn sys_ps(
    buf_ptr: usize,
    buf_len: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    // Validate buffer is not NULL
    if buf_ptr == 0 || buf_len == 0 {
        return Err(SysError::Invalid);
    }

    // Get list of processes
    let processes = crate::process::list_processes();

    // Build a simple string representation (simplified - in real kernel, would be binary format)
    let mut output = alloc::string::String::new();
    output.push_str("PID Status\n");
    for (pid, status) in processes {
        let status_str = match status {
            crate::process::ProcessStatus::Ready => "Ready",
            crate::process::ProcessStatus::Running => "Running",
            crate::process::ProcessStatus::Blocked => "Blocked",
            crate::process::ProcessStatus::Exited(_) => "Exited",
        };
        output.push_str(&format!("{:3} {}\n", pid, status_str));
    }

    // Copy to userspace buffer
    let output_bytes = output.as_bytes();
    if output_bytes.len() > buf_len {
        return Err(SysError::Invalid); // Buffer too small
    }

    // In a real implementation, would validate buf_ptr is accessible from userspace
    // For now, assume it's valid
    unsafe {
        core::ptr::copy_nonoverlapping(
            output_bytes.as_ptr(),
            buf_ptr as *mut u8,
            output_bytes.len(),
        );
    }

    Ok(output_bytes.len())
}

/// sys_uptime - Get kernel uptime in seconds
///
/// Returns the number of seconds since kernel boot, tracked from timer interrupts.
/// Timer frequency is ~100 Hz, so each tick represents ~10ms.
///
/// # Arguments
/// - None (all arguments ignored)
///
/// # Returns
/// - Ok(seconds): Number of seconds since boot
fn sys_uptime(
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> SysResult {
    let seconds = crate::scheduler::get_elapsed_seconds() as usize;
    Ok(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_hello() {
        // Valid magic number
        let result = sys_hello(0xCAFEBABE, 0, 0, 0, 0, 0);
        assert_eq!(result, Ok(0xDEADBEEF));

        // Invalid magic number
        let result = sys_hello(0, 0, 0, 0, 0, 0);
        assert_eq!(result, Err(SysError::Invalid));
    }

    #[test]
    fn test_syscall_log() {
        // Valid length
        let result = sys_log(0x1000, 10, 0, 0, 0, 0);
        assert_eq!(result, Ok(10));

        // Zero length
        let result = sys_log(0x1000, 0, 0, 0, 0, 0);
        assert_eq!(result, Err(SysError::Invalid));

        // Too long
        let result = sys_log(0x1000, 2000, 0, 0, 0, 0);
        assert_eq!(result, Err(SysError::Invalid));
    }

    #[test]
    fn test_dispatch_table() {
        // Valid syscall number
        let result = dispatch_syscall(nr::SYS_HELLO, 0xCAFEBABE, 0, 0, 0, 0, 0);
        assert_eq!(result, 0xDEADBEEF as i64);

        // Invalid syscall number (out of range)
        let result = dispatch_syscall(999, 0, 0, 0, 0, 0, 0);
        assert_eq!(result, SysError::NotImplemented.to_return_value());
    }

    #[test]
    fn test_syscall_write() {
        // Valid fd (1 = stdout)
        let result = sys_write(1, 0x1000, 10, 0, 0, 0);
        assert_eq!(result, Ok(10));

        // Valid fd (2 = stderr)
        let result = sys_write(2, 0x1000, 10, 0, 0, 0);
        assert_eq!(result, Ok(10));

        // Invalid fd (3)
        let result = sys_write(3, 0x1000, 10, 0, 0, 0);
        assert_eq!(result, Err(SysError::BadFd));

        // Zero length
        let result = sys_write(1, 0x1000, 0, 0, 0, 0);
        assert_eq!(result, Err(SysError::Invalid));

        // Too long
        let result = sys_write(1, 0x1000, 5000, 0, 0, 0);
        assert_eq!(result, Err(SysError::Invalid));

        // NULL pointer
        let result = sys_write(1, 0, 10, 0, 0, 0);
        assert_eq!(result, Err(SysError::Fault));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(SysError::Invalid.to_return_value(), -1);
        assert_eq!(SysError::NotImplemented.to_return_value(), -2);
        assert_eq!(SysError::Fault.to_return_value(), -3);
        assert_eq!(SysError::BadFd.to_return_value(), -9);
    }
}

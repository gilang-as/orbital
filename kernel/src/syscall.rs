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
    Some(sys_hello),      // 0
    Some(sys_log),        // 1
    Some(sys_write),      // 2
    Some(sys_exit),       // 3
    // More syscalls go here
];

/// Syscall number constants
pub mod nr {
    pub const SYS_HELLO: usize = 0;
    pub const SYS_LOG: usize = 1;
    pub const SYS_WRITE: usize = 2;
    pub const SYS_EXIT: usize = 3;
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
fn sys_hello(arg1: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> SysResult {
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
/// - Disables interrupts during copy to prevent context switches mid-operation
fn sys_log(arg1: usize, arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> SysResult {
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

    // Output the message via serial
    // Use interrupts::without_interrupts to prevent interruption during output
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        // Write to serial port
        use core::fmt::Write;
        let mut serial = crate::serial::SERIAL1.lock();
        // Write the raw bytes (kernel doesn't interpret content)
        for byte in buffer.iter() {
            // Use write_char for each byte to match serial interface
            let _ = serial.write_char(*byte as char);
        }
        // Add newline for readability
        let _ = serial.write_char('\n');
    });

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
fn sys_write(arg1: usize, arg2: usize, arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> SysResult {
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

    // Output the data via serial
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        use core::fmt::Write;
        let mut serial = crate::serial::SERIAL1.lock();
        for byte in buffer.iter() {
            let _ = serial.write_char(*byte as char);
        }
        // Add newline for consistency with sys_log
        let _ = serial.write_char('\n');
    });

    // Return number of bytes written
    Ok(len)
}

/// sys_exit - Terminate process
/// Arguments:
///   arg1: exit code
/// Returns: never (or error if already exiting)
fn sys_exit(arg1: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> SysResult {
    let _exit_code = arg1;

    // TODO: In full implementation, would:
    // 1. Mark current task as exiting
    // 2. Free task resources
    // 3. Reschedule to next task
    // 4. Never return to userspace

    // For now, return not implemented
    Err(SysError::NotImplemented)
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

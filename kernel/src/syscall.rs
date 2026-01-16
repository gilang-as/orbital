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
    Some(sys_exit),       // 2
    // More syscalls go here
];

/// Syscall number constants
pub mod nr {
    pub const SYS_HELLO: usize = 0;
    pub const SYS_LOG: usize = 1;
    pub const SYS_EXIT: usize = 2;
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
/// Arguments:
///   arg1: pointer to message buffer (unsafe - not validated in this stub)
///   arg2: message length (in bytes)
/// Returns: number of bytes written
fn sys_log(arg1: usize, arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> SysResult {
    let _ptr = arg1;
    let len = arg2;

    // TODO: In full implementation, would:
    // 1. Validate that ptr points to user memory
    // 2. Validate that ptr + len doesn't exceed user memory bounds
    // 3. Read message from user buffer
    // 4. Write to kernel log

    // For now, just return the length as success indicator
    if len == 0 {
        Err(SysError::Invalid)
    } else if len > 1024 {
        // Reasonable limit
        Err(SysError::Invalid)
    } else {
        // In real implementation, would write the message
        // For stub, just return length written
        Ok(len)
    }
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
    fn test_error_codes() {
        assert_eq!(SysError::Invalid.to_return_value(), -1);
        assert_eq!(SysError::NotImplemented.to_return_value(), -2);
        assert_eq!(SysError::Fault.to_return_value(), -3);
    }
}

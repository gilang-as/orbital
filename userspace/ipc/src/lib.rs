//! IPC (Inter-Process Communication) library for Orbital OS
//!
//! This crate provides:
//! 1. Userspace syscall wrappers (safe interfaces to kernel syscalls)
//! 2. Wrappers around the kernel's minimal IPC primitive
//!
//! The kernel provides ONLY a ring buffer for passing raw bytes. Userspace is responsible for:
//! - Message serialization/deserialization
//! - Protocol definition and versioning
//! - Routing messages to the right recipients
//! - Access control and capabilities
//! - Retry logic and backpressure
//! - Message ordering guarantees
//!
//! This separation ensures the kernel remains minimal and policies remain in userspace.

use orbital_common::ipc::{MgmtCommand, MgmtResponse, RawIpcMessage};

// ============================================================================
// Syscall Wrappers
// ============================================================================

/// Error type for syscall operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallError {
    Invalid,
    NotImplemented,
    Fault,
    PermissionDenied,
    NotFound,
    Error,
    BadFd,
}

impl SyscallError {
    /// Convert from raw syscall return value
    pub fn from_return_value(val: i64) -> Option<Self> {
        match val {
            -1 => Some(SyscallError::Invalid),
            -2 => Some(SyscallError::NotImplemented),
            -3 => Some(SyscallError::Fault),
            -4 => Some(SyscallError::PermissionDenied),
            -5 => Some(SyscallError::NotFound),
            -6 => Some(SyscallError::Error),
            -9 => Some(SyscallError::BadFd),
            _ => None,
        }
    }
}

/// Result type for syscall operations
pub type SyscallResult<T> = Result<T, SyscallError>;

// Note: These are stubs. In a real implementation, they would invoke
// the actual syscall instruction using inline assembly.
// Format: syscall instruction with:
//   RAX = syscall number
//   RDI, RSI, RDX, RCX, R8, R9 = arguments
//   Return value in RAX

/// Syscall: hello - Test syscall
/// Arguments: magic number (0xCAFEBABE for success)
/// Returns: 0xDEADBEEF on success
pub fn syscall_hello(magic: u64) -> SyscallResult<u64> {
    // TODO: Implement with inline assembly:
    // unsafe {
    //     let result: i64;
    //     asm!("syscall",
    //         inout("rax") 0usize => result,  // syscall number 0
    //         in("rdi") magic,
    //         clobber_abi("C"),
    //     );
    //     if result >= 0 {
    //         Ok(result as u64)
    //     } else {
    //         Err(SyscallError::from_return_value(result).unwrap())
    //     }
    // }

    // Stub: return error for now
    Err(SyscallError::NotImplemented)
}

/// Syscall: log - Write message to kernel log
/// Arguments:
///   ptr: pointer to message buffer
///   len: message length in bytes
/// Returns: number of bytes written on success, error code on failure
pub fn syscall_log(ptr: *const u8, len: usize) -> SyscallResult<usize> {
    // Invoke syscall 1 (SYS_LOG) with:
    //   RAX = 1 (syscall number)
    //   RDI = ptr (first argument)
    //   RSI = len (second argument)
    //
    // Return value in RAX (negative = error, positive = bytes written)

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 1_i64 => result,  // syscall number 1 (SYS_LOG)
            in("rdi") ptr,                  // first argument: pointer
            in("rsi") len,                  // second argument: length
            clobber_abi("C"),               // Tell compiler C calling convention is clobbered
        );

        if result >= 0 {
            Ok(result as usize)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Non-x86_64 platforms: return not implemented
        Err(SyscallError::NotImplemented)
    }
}

/// Syscall: write - Write to file descriptor
/// Arguments:
///   fd: file descriptor (1=stdout, 2=stderr)
///   ptr: pointer to data buffer
///   len: number of bytes to write
/// Returns: number of bytes written on success, error code on failure
pub fn syscall_write(fd: i32, ptr: *const u8, len: usize) -> SyscallResult<usize> {
    // Invoke syscall 2 (SYS_WRITE) with:
    //   RAX = 2 (syscall number)
    //   RDI = fd (file descriptor)
    //   RSI = ptr (pointer to data)
    //   RDX = len (length in bytes)

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 2_i64 => result,  // syscall number 2 (SYS_WRITE)
            in("rdi") fd as usize,          // first argument: fd
            in("rsi") ptr,                  // second argument: pointer
            in("rdx") len,                  // third argument: length
            clobber_abi("C"),               // Tell compiler C calling convention is clobbered
        );

        if result >= 0 {
            Ok(result as usize)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Non-x86_64 platforms: return not implemented
        Err(SyscallError::NotImplemented)
    }
}

/// Syscall: read - Read from file descriptor
/// Arguments:
///   fd: file descriptor (0=stdin)
///   ptr: pointer to buffer to fill
///   len: number of bytes to read
/// Returns: number of bytes read on success, error code on failure
pub fn syscall_read(fd: i32, ptr: *mut u8, len: usize) -> SyscallResult<usize> {
    // Invoke syscall 4 (SYS_READ) with:
    //   RAX = 4 (syscall number)
    //   RDI = fd (file descriptor)
    //   RSI = ptr (pointer to buffer)
    //   RDX = len (length in bytes)

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 4_i64 => result,  // syscall number 4 (SYS_READ)
            in("rdi") fd as usize,          // first argument: fd
            in("rsi") ptr,                  // second argument: pointer
            in("rdx") len,                  // third argument: length
            clobber_abi("C"),               // Tell compiler C calling convention is clobbered
        );

        if result >= 0 {
            Ok(result as usize)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Non-x86_64 platforms: return not implemented
        Err(SyscallError::NotImplemented)
    }
}

/// Syscall: exit - Terminate process
/// Arguments: exit_code
/// Returns: never (or error if already exiting)
pub fn syscall_exit(exit_code: i32) -> SyscallResult<!> {
    // TODO: Implement with inline assembly:
    // unsafe {
    //     let result: i64;
    //     asm!("syscall",
    //         inout("rax") 2usize => result,  // syscall number 2
    //         in("rdi") exit_code,
    //         clobber_abi("C"),
    //     );
    //     if result >= 0 {
    //         unreachable!("exit syscall should never return")
    //     } else {
    //         Err(SyscallError::from_return_value(result).unwrap())?
    //     }
    // }

    // Stub: panic for now
    panic!("exit syscall not yet implemented")
}

/// Syscall: task_create - Create a new process/task
///
/// Creates a lightweight process/task managed by the kernel.
/// Arguments: entry_point (function address)
/// Returns: process ID (positive) on success, error otherwise
pub fn syscall_task_create(entry_point: usize) -> SyscallResult<u64> {
    // Invoke syscall 5 (SYS_TASK_CREATE) with:
    //   RAX = 5 (syscall number)
    //   RDI = entry_point (task entry point address)

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 5_i64 => result,  // syscall number 5 (SYS_TASK_CREATE)
            in("rdi") entry_point,          // first argument: entry point
            clobber_abi("C"),               // Tell compiler C calling convention is clobbered
        );

        if result >= 0 {
            Ok(result as u64)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Non-x86_64 platforms: return not implemented
        Err(SyscallError::NotImplemented)
    }
}

/// Syscall: task_wait - Wait for a task to complete
///
/// Blocks until the specified task exits, returning its exit code.
/// Arguments: task_id (process ID to wait for)
/// Returns: exit code on success, error otherwise
pub fn syscall_task_wait(task_id: u64) -> SyscallResult<i64> {
    // Invoke syscall 6 (SYS_TASK_WAIT) with:
    //   RAX = 6 (syscall number)
    //   RDI = task_id (process ID to wait for)

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 6_i64 => result,  // syscall number 6 (SYS_TASK_WAIT)
            in("rdi") task_id,              // first argument: task ID
            clobber_abi("C"),               // Tell compiler C calling convention is clobbered
        );

        if result >= 0 {
            Ok(result as i64)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Non-x86_64 platforms: return not implemented
        Err(SyscallError::NotImplemented)
    }
}

/// Syscall: get_pid - Get current process ID
///
/// Returns the ID of the currently running task.
/// Useful for tasks to identify themselves.
/// Returns: process ID (positive)
pub fn syscall_get_pid() -> SyscallResult<u64> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 7_i64 => result,  // syscall number 7 (SYS_GET_PID)
            clobber_abi("C"),
        );

        if result >= 0 {
            Ok(result as u64)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        Err(SyscallError::NotImplemented)
    }
}

/// Syscall: ps - List all processes
///
/// Writes process list to buffer in kernel.
/// Buffer format: "PID Status\n" for each process
/// Returns: number of bytes written
pub fn syscall_ps(buffer: &mut [u8]) -> SyscallResult<usize> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 8_i64 => result,  // syscall number 8 (SYS_PS)
            in("rdi") buffer.as_mut_ptr(),
            in("rsi") buffer.len(),
            clobber_abi("C"),
        );

        if result >= 0 {
            Ok(result as usize)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        Err(SyscallError::NotImplemented)
    }
}

/// Syscall: uptime - Get kernel uptime in seconds
///
/// Returns the number of seconds since kernel boot.
/// Useful for performance measurement and debugging.
/// Returns: uptime in seconds
pub fn syscall_uptime() -> SyscallResult<u64> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 9_i64 => result,  // syscall number 9 (SYS_UPTIME)
            clobber_abi("C"),
        );

        if result >= 0 {
            Ok(result as u64)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        Err(SyscallError::NotImplemented)
    }
}

/// syscall_clear_screen - Clear the VGA display
/// 
/// Clears the entire screen by invoking sys_clear_screen.
/// 
/// # Returns
/// - Ok(()): Success
/// - Err(SyscallError): If syscall failed
pub fn syscall_clear_screen() -> SyscallResult<()> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 10_i64 => result,  // syscall number 10 (SYS_CLEAR_SCREEN)
            clobber_abi("C"),
        );

        if result >= 0 {
            Ok(())
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        Err(SyscallError::NotImplemented)
    }
}

/// syscall_run_ready - Execute all ready processes
/// 
/// Runs all processes currently in the Ready state, executing them synchronously.
/// This is used by the userspace shell's `run` command.
/// 
/// # Returns
/// - Ok(count): Number of processes executed
/// - Err(SyscallError): If syscall failed
pub fn syscall_run_ready() -> SyscallResult<usize> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inout("rax") 11_i64 => result,  // syscall number 11 (SYS_RUN_READY)
            clobber_abi("C"),
        );

        if result >= 0 {
            Ok(result as usize)
        } else {
            Err(SyscallError::from_return_value(result).unwrap_or(SyscallError::Error))
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        Err(SyscallError::NotImplemented)
    }
}

/// Protocol version for IPC messages
pub const IPC_PROTOCOL_VERSION: u32 = 1;

/// IPC client for sending commands to the management daemon
///
/// This wraps the kernel's ring buffer and handles:
/// - Serialization of MgmtCommand to bytes
/// - Protocol versioning
/// - Error handling and retries
pub struct IpcClient {
    /// Task ID of this process (would come from kernel in real implementation)
    task_id: u32,
    /// Message counter for tracking (userspace-defined)
    msg_counter: u32,
}

impl IpcClient {
    /// Create a new IPC client with the given task ID
    pub fn new(task_id: u32) -> Self {
        IpcClient {
            task_id,
            msg_counter: 0,
        }
    }

    /// Serialize a MgmtCommand to bytes
    ///
    /// Format (userspace-defined):
    /// [0] = command type (0=GetState, 1=Shutdown)
    fn serialize_command(cmd: MgmtCommand) -> [u8; 4] {
        let mut bytes = [0u8; 4];
        bytes[0] = match cmd {
            MgmtCommand::GetState => 0,
            MgmtCommand::Shutdown => 1,
        };
        bytes
    }

    /// Deserialize response bytes to MgmtResponse
    ///
    /// Format (userspace-defined):
    /// [0] = response type (0=Ok, 1=Error)
    fn deserialize_response(bytes: &[u8]) -> MgmtResponse {
        if bytes.is_empty() {
            return MgmtResponse::Error;
        }
        match bytes[0] {
            0 => MgmtResponse::Ok,
            1 => MgmtResponse::Error,
            _ => MgmtResponse::Error,
        }
    }

    /// Send a command to the management daemon
    ///
    /// Note: This is a stub. In a real implementation, this would:
    /// 1. Access the kernel's shared ring buffer
    /// 2. Serialize the command
    /// 3. Place it in the ring buffer
    /// 4. Wait for response on another ring buffer
    pub fn send_command(&mut self, cmd: MgmtCommand) -> Result<MgmtResponse, &'static str> {
        self.msg_counter += 1;

        // Create message with serialized command
        let payload = Self::serialize_command(cmd);
        let mut msg = RawIpcMessage {
            sender_task_id: self.task_id,
            msg_id: self.msg_counter,
            payload_len: 4,
            payload: [0u8; 256],
        };
        msg.payload[..4].copy_from_slice(&payload);

        // In a real implementation, would send via kernel ring buffer
        // For now, return a stub response
        Ok(MgmtResponse::Ok)
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new(0)
    }
}

/// IPC server for the management daemon
///
/// This wraps the kernel's ring buffer and handles:
/// - Deserialization of bytes to MgmtCommand
/// - Protocol versioning checks
/// - Routing commands to handlers
pub struct IpcServer {
    /// Task ID of the management daemon
    task_id: u32,
}

impl IpcServer {
    /// Create a new IPC server with the given task ID
    pub fn new(task_id: u32) -> Self {
        IpcServer { task_id }
    }

    /// Wait for the next incoming command
    ///
    /// Note: This is a stub. In a real implementation, this would:
    /// 1. Read from the kernel's shared ring buffer
    /// 2. Check protocol version
    /// 3. Deserialize the command
    /// 4. Return the parsed MgmtCommand
    pub fn accept_command(&mut self) -> Option<MgmtCommand> {
        // In a real implementation, would read from kernel ring buffer
        // and deserialize using deserialization logic
        None
    }

    /// Send a response to the caller
    ///
    /// Note: This is a stub. Would use kernel ring buffer in real implementation.
    pub fn send_response(&self, _msg_id: u32, response: MgmtResponse) -> Result<(), &'static str> {
        let payload = match response {
            MgmtResponse::Ok => [0u8; 1],
            MgmtResponse::Error => [1u8; 1],
        };

        let mut msg = RawIpcMessage {
            sender_task_id: self.task_id,
            msg_id: 0,
            payload_len: 1,
            payload: [0u8; 256],
        };
        msg.payload[0] = payload[0];

        Ok(())
    }
}

impl Default for IpcServer {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new() {
        let client = IpcClient::new(42);
        assert_eq!(client.task_id, 42);
        assert_eq!(client.msg_counter, 0);
    }

    #[test]
    fn test_serialize_command() {
        let bytes_getstate = IpcClient::serialize_command(MgmtCommand::GetState);
        assert_eq!(bytes_getstate[0], 0);

        let bytes_shutdown = IpcClient::serialize_command(MgmtCommand::Shutdown);
        assert_eq!(bytes_shutdown[0], 1);
    }

    #[test]
    fn test_deserialize_response() {
        let ok_bytes = [0u8; 4];
        assert!(matches!(
            IpcClient::deserialize_response(&ok_bytes),
            MgmtResponse::Ok
        ));

        let err_bytes = [1u8; 4];
        assert!(matches!(
            IpcClient::deserialize_response(&err_bytes),
            MgmtResponse::Error
        ));
    }

    #[test]
    fn test_server_new() {
        let server = IpcServer::new(1);
        assert_eq!(server.task_id, 1);
    }
}

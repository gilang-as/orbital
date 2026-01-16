//! IPC (Inter-Process Communication) library for Orbital OS
//!
//! This crate provides userspace wrappers around the kernel's minimal IPC primitive.
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

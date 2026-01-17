#![no_std]

//! Shared types and interfaces for Orbital OS
//!
//! This crate contains types used across kernel, userspace, and IPC boundaries.
//! No implementation logic belongs here - only definitions.

/// IPC message types
pub mod ipc {
    /// Raw IPC message transmitted over ring buffer
    ///
    /// This is a minimal byte-oriented message type. Userspace is responsible
    /// for interpreting the payload - kernel only moves bytes.
    #[derive(Debug, Clone, Copy)]
    pub struct RawIpcMessage {
        /// Sender task ID (set by kernel)
        pub sender_task_id: u32,
        /// Message ID (application-defined)
        pub msg_id: u32,
        /// Payload size in bytes (0-256)
        pub payload_len: u16,
        /// Message payload (userspace-defined format)
        pub payload: [u8; 256],
    }

    /// IPC message header (metadata only, no payload interpretation)
    #[derive(Debug, Clone, Copy)]
    pub struct IpcMessageHeader {
        /// Protocol version for compatibility checking
        pub version: u32,
        /// Sender task ID
        pub sender_task_id: u32,
        /// Unique message identifier
        pub msg_id: u32,
        /// Payload size
        pub payload_len: u16,
    }

    /// A command sent to the management daemon
    #[derive(Debug, Clone, Copy)]
    pub enum MgmtCommand {
        /// Request the system state
        GetState,
        /// Shutdown the system
        Shutdown,
    }

    /// Response from the management daemon
    #[derive(Debug, Clone, Copy)]
    pub enum MgmtResponse {
        /// Operation successful
        Ok,
        /// Operation failed
        Error,
    }
}

/// Error types
#[derive(Debug, Clone, Copy)]
pub enum OrbitalError {
    /// IPC communication failed
    IpcError,
    /// Invalid configuration
    ConfigError,
    /// Permission denied
    PermissionDenied,
}

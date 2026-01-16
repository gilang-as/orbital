#![no_std]

//! Shared types and interfaces for Orbital OS
//!
//! This crate contains types used across kernel, userspace, and IPC boundaries.
//! No implementation logic belongs here - only definitions.

/// IPC message types
pub mod ipc {
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

//! IPC Ring Buffer Primitive
//!
//! This module provides the ONLY IPC abstraction in the kernel:
//! - A shared memory ring buffer for message passing
//! - No serialization logic
//! - No protocol interpretation
//! - No policy enforcement
//!
//! All higher-level concerns (routing, authentication, message formats,
//! protocol versioning) belong in userspace.

use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Maximum payload per message (256 bytes)
pub const MAX_PAYLOAD: usize = 256;

/// Maximum messages in ring buffer (power of 2)
pub const RING_BUFFER_SIZE: usize = 256;

/// Mask for ring buffer index wrapping
const RING_MASK: usize = RING_BUFFER_SIZE - 1;

/// Raw IPC message transmitted over ring buffer.
/// This is a simple byte container - userspace interprets the payload.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RingMessage {
    /// Sender task ID (set by kernel, unused by kernel)
    pub sender_task_id: u32,
    /// Message ID (userspace-defined, unused by kernel)
    pub msg_id: u32,
    /// Payload size in bytes (0-256)
    pub payload_len: u16,
    /// Raw message payload - userspace decides interpretation
    pub payload: [u8; MAX_PAYLOAD],
}

impl RingMessage {
    /// Create a new message with zeroed payload
    pub fn new(sender_task_id: u32, msg_id: u32, payload_len: u16) -> Self {
        debug_assert!(
            payload_len as usize <= MAX_PAYLOAD,
            "payload_len must be <= {}",
            MAX_PAYLOAD
        );
        RingMessage {
            sender_task_id,
            msg_id,
            payload_len,
            payload: [0u8; MAX_PAYLOAD],
        }
    }

    /// Set payload bytes. Caller is responsible for payload_len accuracy.
    pub fn set_payload(&mut self, data: &[u8]) {
        let len = data.len().min(MAX_PAYLOAD);
        self.payload[..len].copy_from_slice(&data[..len]);
        self.payload_len = len as u16;
    }

    /// Get payload as slice
    pub fn payload_slice(&self) -> &[u8] {
        &self.payload[..self.payload_len as usize]
    }
}

/// Ring buffer for IPC messages.
///
/// A simple lock-free ring buffer using atomic indices.
/// Supports single writer, single reader per endpoint.
///
/// DESIGN RATIONALE - Why this minimal approach:
/// - No message serialization: Userspace defines protocol
/// - No message routing: Userspace owns IPC router
/// - No access control: Userspace enforces capabilities
/// - No message framing: Userspace adds headers/checksums
/// - No backpressure: Userspace handles send failures
/// - No timeouts: Userspace implements receive timeouts
/// - No queuing policies: Userspace implements priority/scheduling
pub struct RingBuffer {
    /// Ring buffer storage (allocated in kernel heap)
    /// Uses UnsafeCell for interior mutability with atomic indices
    messages: UnsafeCell<Vec<RingMessage>>,

    /// Write index (incremented by enqueue)
    write_index: AtomicUsize,

    /// Read index (incremented by dequeue)
    read_index: AtomicUsize,
}

// SAFETY: RingBuffer is Send+Sync because:
// - messages are only accessed through atomic index operations
// - each slot is only accessed by one writer and one reader
// - AtomicUsize provides synchronization
unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

impl RingBuffer {
    /// Create a new ring buffer
    pub fn new() -> Self {
        RingBuffer {
            messages: UnsafeCell::new(Vec::new()),
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
        }
    }

    /// Initialize the buffer with empty messages
    pub fn init(&self) {
        // SAFETY: Only initialization, no concurrent access
        unsafe {
            let messages = &mut *self.messages.get();
            messages.clear();
            for _ in 0..RING_BUFFER_SIZE {
                messages.push(RingMessage::new(0, 0, 0));
            }
        }
        self.write_index.store(0, Ordering::Release);
        self.read_index.store(0, Ordering::Release);
    }

    /// Enqueue a message. Returns Ok(()) on success, Err(()) if buffer full.
    ///
    /// KERNEL RESPONSIBILITY:
    /// - Write message bytes to ring buffer
    /// - Update indices atomically
    ///
    /// NOT kernel responsibility (userspace handles):
    /// - Determining which process can send
    /// - Routing the message to correct recipient
    /// - Retrying on failure
    /// - Enforcing size limits per sender
    pub fn enqueue(&self, message: &RingMessage) -> Result<(), ()> {
        let write = self.write_index.load(Ordering::Acquire);
        let read = self.read_index.load(Ordering::Acquire);

        // Check if buffer is full (write + 1 == read)
        if (write + 1) & RING_MASK == read & RING_MASK {
            return Err(());
        }

        // SAFETY: write_index is guaranteed within bounds due to masking
        let idx = write & RING_MASK;
        unsafe {
            let messages = &mut *self.messages.get();
            // Copy message directly into ring buffer
            let dst = &mut messages[idx] as *mut RingMessage as *mut u8;
            let src = message as *const RingMessage as *const u8;
            core::ptr::copy_nonoverlapping(src, dst, size_of::<RingMessage>());
        }

        // Update write index with full barrier
        self.write_index
            .store((write + 1) & 0xFFFFFFFF, Ordering::Release);

        Ok(())
    }

    /// Dequeue a message. Returns Some(message) if available, None if empty.
    ///
    /// KERNEL RESPONSIBILITY:
    /// - Read message bytes from ring buffer
    /// - Update indices atomically
    ///
    /// NOT kernel responsibility (userspace handles):
    /// - Determining which process can receive from this endpoint
    /// - Filtering messages by sender/type
    /// - Waiting for messages (blocking/polling policy)
    /// - Message ordering guarantees across endpoints
    pub fn dequeue(&self) -> Option<RingMessage> {
        let read = self.read_index.load(Ordering::Acquire);
        let write = self.write_index.load(Ordering::Acquire);

        // Check if buffer is empty
        if read & RING_MASK == write & RING_MASK {
            return None;
        }

        // SAFETY: read_index is guaranteed within bounds due to masking
        let idx = read & RING_MASK;
        let message = unsafe {
            let messages = &*self.messages.get();
            // Copy message directly from ring buffer
            let src = &messages[idx] as *const RingMessage;
            core::ptr::read(src)
        };

        // Update read index with full barrier
        self.read_index
            .store((read + 1) & 0xFFFFFFFF, Ordering::Release);

        Some(message)
    }

    /// Check if buffer is empty (no guarantee on concurrent systems)
    pub fn is_empty(&self) -> bool {
        let read = self.read_index.load(Ordering::Acquire);
        let write = self.write_index.load(Ordering::Acquire);
        read & RING_MASK == write & RING_MASK
    }

    /// Get current queue depth (no guarantee on concurrent systems)
    pub fn depth(&self) -> usize {
        let write = self.write_index.load(Ordering::Acquire);
        let read = self.read_index.load(Ordering::Acquire);
        (write - read) & RING_MASK
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        let rb = RingBuffer::new();
        rb.init();

        let mut msg = RingMessage::new(42, 1, 5);
        msg.payload[..5].copy_from_slice(b"hello");

        assert!(rb.enqueue(&msg).is_ok());
        assert_eq!(rb.depth(), 1);

        let received = rb.dequeue();
        assert!(received.is_some());
        let received = received.unwrap();
        assert_eq!(received.sender_task_id, 42);
        assert_eq!(received.msg_id, 1);
        assert_eq!(received.payload_len, 5);
        assert_eq!(received.payload_slice(), b"hello");
    }

    #[test]
    fn test_empty() {
        let rb = RingBuffer::new();
        rb.init();
        assert!(rb.is_empty());
        assert_eq!(rb.depth(), 0);
    }
}

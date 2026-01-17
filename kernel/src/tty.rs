//! TTY (Teletype) primitive
//!
//! A minimal terminal device abstraction that decouples logical output (stdout/stderr)
//! from physical backends (serial port, VGA buffer).
//!
//! ## Design Philosophy
//!
//! - **Primitives only**: No buffering, line discipline, or control sequences
//! - **Policy-free**: Kernel writes raw bytes; no interpretation
//! - **Stateless**: Each write is independent; no internal queuing
//! - **Backend-agnostic**: Routes to available output devices
//!
//! ## Current Implementation
//!
//! Routes all writes to the serial port (UART 0x3F8). VGA buffer support exists
//! but is not used by default to avoid display corruption during kernel output.
//!
//! ## Safety
//!
//! - Disables interrupts during write to prevent context switches
//! - Locks serial port mutex during access
//! - No panics on invalid input (caller is responsible for validation)

use core::fmt::Write;
use x86_64::instructions::interrupts;

/// Maximum bytes per TTY write operation
/// Matches sys_write validation limit
const TTY_MAX_WRITE: usize = 4096;

/// Write to TTY device
///
/// Routes raw bytes to configured output backend (currently serial port).
/// Does not add newlines or modify content.
///
/// # Arguments
///
/// * `buf` - Byte slice to write (must not exceed TTY_MAX_WRITE)
///
/// # Returns
///
/// Number of bytes written on success
///
/// # Panics
///
/// Panics if buffer length exceeds TTY_MAX_WRITE
pub fn tty_write(buf: &[u8]) -> usize {
    if buf.len() > TTY_MAX_WRITE {
        panic!(
            "TTY write exceeds maximum size: {} > {}",
            buf.len(),
            TTY_MAX_WRITE
        );
    }

    if buf.is_empty() {
        return 0;
    }

    // Disable interrupts during write to ensure atomicity
    // This prevents other code from interleaving output
    interrupts::without_interrupts(|| {
        // Get exclusive access to serial port
        let mut serial = crate::serial::SERIAL1.lock();

        // Write each byte directly without modification
        for &byte in buf {
            let _ = serial.write_char(byte as char);
        }
    });

    buf.len()
}

/// Write to TTY with newline (internal use)
///
/// Used by kernel logging to add readability.
/// Appends newline after data.
///
/// # Arguments
///
/// * `buf` - Byte slice to write
pub fn tty_write_with_newline(buf: &[u8]) -> usize {
    let written = tty_write(buf);

    interrupts::without_interrupts(|| {
        let mut serial = crate::serial::SERIAL1.lock();
        let _ = serial.write_char('\n');
    });

    written
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty_write_empty() {
        let result = tty_write(&[]);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_tty_write_single_byte() {
        let data = [b'A'];
        let result = tty_write(&data);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_tty_write_multiple_bytes() {
        let data = b"Hello, World!";
        let result = tty_write(data);
        assert_eq!(result, data.len());
    }

    #[test]
    fn test_tty_write_max_size() {
        let data = alloc::vec![b'x'; TTY_MAX_WRITE];
        let result = tty_write(&data);
        assert_eq!(result, TTY_MAX_WRITE);
    }

    #[test]
    #[should_panic(expected = "exceeds maximum size")]
    fn test_tty_write_exceeds_max() {
        let data = alloc::vec![b'x'; TTY_MAX_WRITE + 1];
        let _ = tty_write(&data);
    }

    #[test]
    fn test_tty_write_with_newline() {
        let data = b"Log message";
        let result = tty_write_with_newline(data);
        assert_eq!(result, data.len());
    }
}

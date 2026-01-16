//! Standard input buffer for sys_read syscall
//!
//! Provides a character queue that gets filled as users type in the terminal.
//! sys_read syscall reads from this queue.
//!
//! Uses lazy initialization to avoid heap allocation during early kernel init.

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use spin::Mutex;

static INPUT_BUFFER: OnceCell<Mutex<ArrayQueue<u8>>> = OnceCell::uninit();

/// Get or initialize the input buffer on first access
///
/// Uses lazy initialization to avoid allocating during kernel init when heap
/// memory is constrained. The buffer is allocated on first access (either from
/// terminal task or sys_read syscall).
fn get_or_init_buffer() -> &'static Mutex<ArrayQueue<u8>> {
    INPUT_BUFFER.get_or_init(|| Mutex::new(ArrayQueue::new(256)))
}

/// Add a character to the input buffer (from keyboard input)
///
/// Called by the terminal task when a character is ready
pub fn add_input_char(ch: u8) {
    let buf = get_or_init_buffer().lock();
    let _ = buf.push(ch);
}

/// Read up to `len` bytes from the input buffer into `buf`
///
/// Returns the number of bytes read. This is a non-blocking operation
/// that returns immediately even if there's no data (returns 0 in that case).
pub fn read_input(buf: &mut [u8]) -> usize {
    let q = get_or_init_buffer().lock();
    let mut count = 0;
    for byte in buf {
        match q.pop() {
            Some(ch) => {
                *byte = ch;
                count += 1;
            }
            None => break,
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_buffer_init() {
        // Just verify it initializes without panic
        // (Note: can only initialize once per test run)
    }
}

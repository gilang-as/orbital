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
static SCANCODE_BUFFER: OnceCell<Mutex<ArrayQueue<u8>>> = OnceCell::uninit();

/// Get or initialize the input buffer on first access
fn get_or_init_buffer() -> &'static Mutex<ArrayQueue<u8>> {
    INPUT_BUFFER.get_or_init(|| Mutex::new(ArrayQueue::new(256)))
}

/// Get or initialize the scancode buffer
fn get_or_init_scancode_buffer() -> &'static Mutex<ArrayQueue<u8>> {
    SCANCODE_BUFFER.get_or_init(|| Mutex::new(ArrayQueue::new(256)))
}

/// Add a character to the input buffer (from keyboard input)
pub fn add_input_char(ch: u8) {
    let buf = get_or_init_buffer().lock();
    let _ = buf.push(ch);
}

/// Add a scancode for terminal_main to read
pub fn add_scancode(scancode: u8) {
    let buf = get_or_init_scancode_buffer().lock();
    let _ = buf.push(scancode);
}

/// Get a scancode if available (non-blocking)
pub fn get_scancode() -> Option<u8> {
    let buf = get_or_init_scancode_buffer().lock();
    buf.pop()
}

/// Read up to `len` bytes from the input buffer into `buf`
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

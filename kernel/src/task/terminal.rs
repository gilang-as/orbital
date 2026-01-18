use crate::task::keyboard::ScancodeStream;
use crate::{print, println};
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

/// Terminal I/O handler - minimal keyboard/display plumbing
/// 
/// This task:
/// 1. Reads keyboard input from hardware
/// 2. Echoes characters to VGA screen for user feedback  
/// 3. Queues input to buffer for userspace to read via sys_read(0)
///
/// This is NOT a shell. All command execution happens in userspace.
pub async fn terminal() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    println!("Kernel I/O Ready - Waiting for userspace CLI...");
    print!("> ");
    update_cursor();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        match character {
                            '\n' => {
                                // Newline: echo to screen, queue to userspace
                                println!();
                                print!("> ");
                                update_cursor();
                                // Queue newline for userspace
                                crate::input::add_input_char(b'\n');
                            }
                            '\u{0008}' => {
                                // Backspace: echo to screen, queue to userspace
                                print!("\u{0008}");
                                update_cursor();
                                crate::input::add_input_char(b'\x08');
                            }
                            _ => {
                                // Regular character: echo to screen, queue to userspace
                                print!("{}", character);
                                update_cursor();
                                crate::input::add_input_char(character as u8);
                            }
                        }
                    }
                    DecodedKey::RawKey(_key) => {
                        // Ignore raw keys
                    }
                }
            }
        }
    }
}

/// Update the VGA hardware cursor position
fn update_cursor() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let writer = crate::vga_buffer::WRITER.lock();
        writer.update_cursor();
        writer.show_cursor();
    });
}

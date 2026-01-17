use crate::{print, println};
use crate::shell::Shell;
use alloc::string::String;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

pub async fn terminal() {
    let mut scancodes = crate::task::keyboard::ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    let mut shell = Shell::new();
    let mut input_line = String::new();
    
    print!("> ");
    update_cursor();

    use futures_util::stream::StreamExt;
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        match character {
                            '\n' => {
                                println!();
                                if !input_line.is_empty() {
                                    shell.execute(&input_line);
                                    input_line.clear();
                                }
                                print!("> ");
                                update_cursor();
                            }
                            '\u{0008}' => { // Backspace
                                if !input_line.is_empty() {
                                    input_line.pop();
                                    // Print actual backspace character - VGA buffer will handle it
                                    print!("\u{0008}");
                                    update_cursor();
                                }
                            }
                            _ => {
                                input_line.push(character);
                                print!("{}", character);
                                update_cursor();
                                
                                // Queue character for userspace to read via sys_read
                                crate::input::add_input_char(character as u8);
                            }
                        }
                    }
                    DecodedKey::RawKey(_key) => {
                        // Ignore raw keys for now
                    }
                }
            }
        }
    }
}

/// Synchronous terminal main function for use as a kernel process
/// This is a blocking version that runs as a kernel process instead of an async task
pub fn terminal_main() -> i64 {
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    let mut shell = Shell::new();
    let mut input_line = String::new();
    
    print!("> ");
    update_cursor();

    loop {
        // Read a scancode from the input queue
        // In a real implementation, this would use sys_read or block properly
        // For now, we do a simple polling approach
        if let Some(scancode) = crate::input::get_scancode() {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => {
                            match character {
                                '\n' => {
                                    println!();
                                    if !input_line.is_empty() {
                                        shell.execute(&input_line);
                                        input_line.clear();
                                    }
                                    print!("> ");
                                    update_cursor();
                                }
                                '\u{0008}' => { // Backspace
                                    if !input_line.is_empty() {
                                        input_line.pop();
                                        print!("\u{0008}");
                                        update_cursor();
                                    }
                                }
                                _ => {
                                    input_line.push(character);
                                    print!("{}", character);
                                    update_cursor();
                                }
                            }
                        }
                        DecodedKey::RawKey(_key) => {
                            // Ignore raw keys
                        }
                    }
                }
            }
        } else {
            // No input available - yield to other processes
            // This is where sys_yield would be called
            let _ = crate::syscall::dispatch_syscall(100, 0, 0, 0, 0, 0, 0); // Placeholder for yield
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

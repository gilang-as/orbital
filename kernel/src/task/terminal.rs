use crate::{print, println};
use crate::shell::Shell;
use alloc::string::String;
use crate::task::keyboard::ScancodeStream;
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

pub async fn terminal() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    let mut shell = Shell::new();
    let mut input_line = String::new();
    
    println!("> ");

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

/// Update the VGA hardware cursor position
fn update_cursor() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let writer = crate::vga_buffer::WRITER.lock();
        writer.update_cursor();
    });
}

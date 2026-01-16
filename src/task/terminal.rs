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
    
    println!("> ", );

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
                            }
                            '\u{0008}' => { // Backspace
                                if !input_line.is_empty() {
                                    input_line.pop();
                                    print!("\u{0008} \u{0008}");
                                }
                            }
                            _ => {
                                input_line.push(character);
                                print!("{}", character);
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

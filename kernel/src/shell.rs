use alloc::vec::Vec;
use crate::println;

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Shell
    }

    pub fn execute(&mut self, command: &str) {
        let trimmed = command.trim();
        
        if trimmed.is_empty() {
            return;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        
        match parts.get(0).copied() {
            Some("echo") => {
                if parts.len() > 1 {
                    let message = parts[1..].join(" ");
                    println!("{}", message);
                }
            }
            Some("help") => {
                println!("Available commands:");
                println!("  echo <message>  - Print a message");
                println!("  help            - Show this help message");
                println!("  clear           - Clear the screen");
            }
            Some("clear") => {
                crate::vga_buffer::clear_screen();
            }
            Some(cmd) => {
                println!("Unknown command: '{}'", cmd);
            }
            None => {}
        }
    }
}

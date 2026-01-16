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
            Some("ping") => {
                println!("pong");
            }
            Some("spawn") => {
                // Demo: spawn a task with a dummy entry point
                // In real usage, this would be replaced with orbital-cli or other userspace process
                let pid = crate::process::create_process(0x1000);
                if pid > 0 {
                    println!("Spawned process with PID: {}", pid);
                    let status = crate::process::get_process_status(pid as u64);
                    println!("Process status: {:?}", status);
                } else {
                    println!("Failed to spawn process: error {}", pid);
                }
            }
            Some("ps") => {
                // List all processes
                let processes = crate::process::list_processes();
                println!("PID\tStatus");
                for (pid, status) in processes {
                    println!("{}\t{:?}", pid, status);
                }
            }
            Some("help") => {
                println!("Available commands:");
                println!("  echo <message>  - Print a message");
                println!("  ping            - Respond with pong");
                println!("  spawn           - Create a new task");
                println!("  ps              - List all processes");
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

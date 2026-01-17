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
                // Spawn a test task by index: spawn 1, spawn 2, etc.
                // Default to task 1 if no argument given
                let task_index = parts.get(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                
                if let Some(task_fn) = crate::tasks::get_test_task(task_index) {
                    let pid = crate::process::create_process(task_fn as usize);
                    if pid > 0 {
                        println!("Spawned task {} with PID: {}", task_index, pid);
                    } else {
                        println!("Failed to spawn task {}: error {}", task_index, pid);
                    }
                } else {
                    println!("Unknown task index: {}. Try: spawn 1, spawn 2, spawn 3, spawn 4", task_index);
                }
            }
            Some("run") => {
                // Execute all ready processes
                println!("Executing all ready processes...");
                let count = crate::process::execute_all_ready();
                println!("Executed {} processes", count);
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
                println!("  run             - Execute all ready tasks");
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

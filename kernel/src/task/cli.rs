/// Shell task - reads input and executes commands
/// 
/// This task:
/// 1. Reads lines from input buffer via keyboard
/// 2. Parses commands
/// 3. Executes command logic
///
/// This is POLICY - command dispatch and execution.
/// The terminal task (I/O only) feeds input to this task.

use alloc::string::String;
use crate::{print, println};

pub async fn shell() {
    println!("╔════════════════════════════════════════╗");
    println!("║       Orbital OS - Shell v0.1.0        ║");
    println!("║    Type 'help' for available commands  ║");
    println!("╚════════════════════════════════════════╝");

    loop {
        // Read a line from keyboard input buffer
        let line = read_line().await;
        
        if line.is_empty() {
            continue;
        }

        execute_command(&line);
    }
}

/// Read a line from the input buffer (blocks until newline)
async fn read_line() -> String {
    let mut line = String::new();
    
    loop {
        // Read from input buffer
        let mut buf = [0u8; 256];
        let n = crate::input::read_input(&mut buf);
        
        if n > 0 {
            for i in 0..n {
                let ch = buf[i] as char;
                match ch {
                    '\n' => {
                        return line;
                    }
                    '\u{0008}' => {
                        // Backspace
                        line.pop();
                    }
                    _ => {
                        line.push(ch);
                    }
                }
            }
        }
        
        // Yield to other tasks
        crate::task::keyboard::ScancodeStream::new();
    }
}

/// Execute a shell command
fn execute_command(command: &str) {
    let parts: alloc::vec::Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => cmd_help(),
        "echo" => cmd_echo(&parts[1..]),
        "ps" => cmd_ps(),
        "pid" => cmd_pid(),
        "uptime" => cmd_uptime(),
        "ping" => cmd_ping(),
        "spawn" => cmd_spawn(&parts[1..]),
        "wait" => cmd_wait(&parts[1..]),
        "run" => cmd_run(),
        "clear" => cmd_clear(),
        "exit" => cmd_exit(),
        _ => println!("unknown command: '{}' (try 'help')", parts[0]),
    }
}

fn cmd_help() {
    println!("Available commands:");
    println!("  help            - Show this help");
    println!("  echo <text>     - Echo text");
    println!("  ps              - List processes");
    println!("  pid             - Show current PID");
    println!("  uptime          - Show kernel uptime");
    println!("  ping            - Connectivity test");
    println!("  spawn <n>       - Spawn n tasks");
    println!("  wait <pid>      - Wait for process");
    println!("  run             - Execute ready tasks");
    println!("  clear           - Clear screen");
    println!("  exit            - Exit shell");
}

fn cmd_echo(args: &[&str]) {
    if args.is_empty() {
        println!("");
    } else {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 { print!(" "); }
            print!("{}", arg);
        }
        println!();
    }
}

fn cmd_ps() {
    println!("Running processes (stub - use ps syscall for details)");
}

fn cmd_pid() {
    let pid = crate::scheduler::current_process().unwrap_or(0);
    println!("Current PID: {}", pid);
}

fn cmd_uptime() {
    let uptime_s = crate::scheduler::get_elapsed_seconds();
    println!("Uptime: {} seconds", uptime_s);
}

fn cmd_ping() {
    println!("pong");
}

fn cmd_spawn(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: spawn <count>");
        return;
    }
    
    if let Ok(count) = args[0].parse::<usize>() {
        for i in 0..count {
            if let Some(entry) = crate::tasks::get_test_task(((i % 4) + 1) as usize) {
                let pid = crate::process::create_process(entry as usize);
                if pid > 0 {
                    crate::scheduler::enqueue_process(pid as u64);
                    println!("Spawned task {}: PID {}", i + 1, pid);
                }
            }
        }
    }
}

fn cmd_wait(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: wait <pid>");
        return;
    }
    
    if let Ok(pid) = args[0].parse::<u64>() {
        if pid > 0 {
            println!("Waiting for PID {}...", pid);
            // TODO: Implement actual wait
            println!("Process completed");
        }
    }
}

fn cmd_run() {
    println!("Executing all ready processes...");
    let count = crate::process::execute_all_ready();
    println!("Executed {} process(es)", count);
}

fn cmd_clear() {
    crate::vga_buffer::clear_screen();
}

fn cmd_exit() {
    println!("Exiting...");
    crate::hlt_loop();
}


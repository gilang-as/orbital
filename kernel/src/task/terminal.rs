use crate::task::keyboard::ScancodeStream;
use crate::{print, println};
use alloc::string::String;
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

/// Terminal task - minimal I/O with embedded CLI for now
/// 
/// This task:
/// 1. Reads keyboard input from hardware
/// 2. Echoes characters to VGA screen for user feedback  
/// 3. **Executes shell commands** (embedded here temporarily)
/// 4. Queues input to buffer for backward compatibility
///
/// TODO: Once we have proper userspace task loading, move command execution to userspace.
pub async fn terminal() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    println!("╔════════════════════════════════════════╗");
    println!("║       Orbital OS - CLI v0.1.0          ║");
    println!("║    Type 'help' for available commands  ║");
    println!("╚════════════════════════════════════════╝");
    print!("> ");
    update_cursor();

    let mut input_line = String::new();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        match character {
                            '\n' => {
                                println!();
                                if !input_line.is_empty() {
                                    execute_command(&input_line);
                                    input_line.clear();
                                }
                                print!("> ");
                                update_cursor();
                                crate::input::add_input_char(b'\n');
                            }
                            '\u{0008}' => {
                                if !input_line.is_empty() {
                                    print!("\u{0008}");
                                    update_cursor();
                                    input_line.pop();
                                }
                                crate::input::add_input_char(b'\x08');
                            }
                            _ => {
                                print!("{}", character);
                                update_cursor();
                                input_line.push(character);
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
        _ => println!("Unknown command: '{}' (try 'help')", parts[0]),
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
    println!("  exit            - Exit CLI");
}

fn cmd_echo(args: &[&str]) {
    if args.is_empty() {
        println!("");
    } else {
        for arg in args {
            print!("{} ", arg);
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
    
    let count: usize = args[0].parse().unwrap_or(1);
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

fn cmd_wait(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: wait <pid>");
        return;
    }
    
    let pid: u64 = args[0].parse().unwrap_or(0);
    if pid > 0 {
        println!("Waiting for PID {}...", pid);
        // TODO: Implement actual wait
        println!("Process completed");
    }
}

fn cmd_run() {
    println!("Executing all ready processes...");
    let count = crate::process::execute_all_ready();
    println!("Executed {} process(es)", count);
}

fn cmd_clear() {
    crate::vga_buffer::clear_screen();
    print!("> ");
    update_cursor();
}

fn cmd_exit() {
    println!("Exiting...");
    crate::hlt_loop();
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

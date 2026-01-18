//! Orbital CLI - Interactive Command-Line Interface
//!
//! Demonstrates userspace policy logic via syscalls.
//! All I/O goes through kernel syscalls:
//! - sys_read(0) for stdin
//! - sys_write(1) for stdout
//! - sys_write(2) for stderr
//!
//! Commands:
//! - help: Display available commands
//! - echo <text>: Echo text to stdout
//! - ps: List running processes
//! - uptime: Show kernel uptime
//! - spawn <count>: Spawn N tasks
//! - exit: Quit the CLI
//!
//! This shows the "policy-free kernel" principle:
//! Kernel provides I/O syscalls, userspace provides command logic.

use orbital_ipc::{syscall_task_create, syscall_task_wait, syscall_write, 
                   syscall_get_pid, syscall_ps, syscall_uptime, 
                   syscall_clear_screen, syscall_run_ready};

// ============================================================================
// Syscall Wrappers
// ============================================================================

/// Invoke sys_read syscall (fd=0 is stdin)
/// 
/// Reads up to `len` bytes from stdin into `buf`
/// Returns number of bytes read
#[inline]
fn syscall_read(fd: i32, buf: *mut u8, len: usize) -> Result<usize, i64> {
    #[cfg(target_arch = "x86_64")]
    {
        let result: i64;
        unsafe {
            std::arch::asm!(
                "syscall",
                inout("rax") 4i64 => result,  // syscall #4 = SYS_READ
                in("rdi") fd as usize,
                in("rsi") buf,
                in("rdx") len,
                clobber_abi("C"),
            );
        }
        
        if result < 0 {
            Err(result)
        } else {
            Ok(result as usize)
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (fd, buf, len);
        Err(-2)  // NotImplemented on non-x86_64
    }
}

/// Invoke sys_write syscall (fd=1 is stdout, fd=2 is stderr)
/// 
/// Writes `len` bytes from `ptr` to file descriptor
#[inline]
fn syscall_write(fd: i32, ptr: *const u8, len: usize) -> Result<usize, i64> {
    #[cfg(target_arch = "x86_64")]
    {
        let result: i64;
        unsafe {
            std::arch::asm!(
                "syscall",
                inout("rax") 2i64 => result,  // syscall #2 = SYS_WRITE
                in("rdi") fd as usize,
                in("rsi") ptr,
                in("rdx") len,
                clobber_abi("C"),
            );
        }
        
        if result < 0 {
            Err(result)
        } else {
            Ok(result as usize)
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (fd, ptr, len);
        Ok(0)
    }
}

// Helper functions for output
fn print(s: &str) {
    let _ = syscall_write(1, s.as_ptr(), s.len());
}

fn println(s: &str) {
    let _ = syscall_write(1, s.as_ptr(), s.len());
    let _ = syscall_write(1, "\n".as_ptr(), 1);
}

// ============================================================================
// CLI Implementation
// ============================================================================

/// Command dispatcher - parses and executes commands
struct Cli;

impl Cli {
    /// Main CLI loop - read commands and execute them
    fn run() {
        Self::print_welcome();

        let mut input_buffer = [0u8; 256];
        let mut running = true;

        while running {
            // Print prompt
            print("> ");

            // Read input line from stdin (fd=0)
            match syscall_read(0, input_buffer.as_mut_ptr(), input_buffer.len()) {
                Ok(n) => {
                    if n == 0 {
                        continue;  // Empty line
                    }

                    // Convert bytes to string (ignore trailing newline)
                    let input_str = match std::str::from_utf8(&input_buffer[..n]) {
                        Ok(s) => s.trim(),
                        Err(_) => {
                            println("Error: Invalid UTF-8 input");
                            continue;
                        }
                    };

                    if input_str.is_empty() {
                        continue;
                    }

                    // Execute command
                    running = Self::execute(input_str);
                }
                Err(e) => {
                    let msg = format!("Error reading input: {}\n", e);
                    println(&msg);
                }
            }
        }

        println("Goodbye!");
    }

    /// Parse and execute a command string
    /// Returns false if user wants to exit
    fn execute(input: &str) -> bool {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return true;
        }

        // Split command and arguments
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return true;
        }

        let command = parts[0];
        let args = &parts[1..];

        match command {
            "help" => Self::cmd_help(),
            "echo" => Self::cmd_echo(args),
            "ps" => Self::cmd_ps(),
            "uptime" => Self::cmd_uptime(),
            "pid" => Self::cmd_pid(),
            "spawn" => Self::cmd_spawn(args),
            "wait" => Self::cmd_wait(args),
            "ping" => Self::cmd_ping(),
            "run" => Self::cmd_run(),
            "clear" => Self::cmd_clear(),
            "exit" | "quit" => return false,
            _ => Self::cmd_unknown(command),
        }

        true
    }

    /// help command - display available commands
    fn cmd_help() {
        println("Available Commands:");
        println("  help              - Show this help message");
        println("  echo <text>       - Echo text to stdout");
        println("  ps                - List running processes (formatted)");
        println("  uptime            - Show kernel uptime");
        println("  pid               - Show current process ID");
        println("  ping              - Test connectivity (responds with pong)");
        println("  spawn <N>         - Spawn task by index (1-4)");
        println("  spawn -c <N>      - Spawn N identical tasks");
        println("  wait <PID>        - Wait for a task to complete (get exit code)");
        println("  run               - Execute all ready processes");
        println("  clear             - Clear the screen");
        println("  exit or quit      - Exit the CLI");
        println("");
        println("Examples:");
        println("  > echo Hello World");
        println("  > ps");
        println("  > spawn 1        (spawn task 1)");
        println("  > wait 1         (wait for PID 1 to complete)");
        println("  > spawn -c 3     (spawn 3 identical tasks)");
        println("  > run            (execute ready tasks)");
    }

    /// echo command - echo arguments to stdout
    fn cmd_echo(args: &[&str]) {
        if args.is_empty() {
            println("");
            return;
        }

        // Reconstruct argument string with spaces
        let mut output = String::new();
        for (i, arg) in args.iter().enumerate() {
            output.push_str(arg);
            if i < args.len() - 1 {
                output.push(' ');
            }
        }
        println(&output);
    }

    /// Unknown command handler
    fn cmd_unknown(cmd: &str) {
        let msg = format!("unknown command: '{}' (try 'help')", cmd);
        println(&msg);
    }

    /// ps command - list running processes with details
    fn cmd_ps() {
        println("Running processes:");
        
        let mut buffer = [0u8; 512];
        match syscall_ps(&mut buffer) {
            Ok(bytes_written) => {
                if let Ok(ps_output) = std::str::from_utf8(&buffer[..bytes_written]) {
                    // Parse and format the output
                    let lines: Vec<&str> = ps_output.lines().collect();
                    
                    if lines.len() > 1 {
                        // Print header with better formatting
                        println("┌─────┬──────────────┐");
                        println("│ PID │ Status       │");
                        println("├─────┼──────────────┤");
                        
                        // Print each process
                        for line in &lines[1..] {
                            if !line.is_empty() {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    let pid = parts[0];
                                    let status = parts[1];
                                    let status_padded = format!("{:<12}", status);
                                    let msg = format!("│ {:3} │ {} │", pid, status_padded);
                                    println(&msg);
                                }
                            }
                        }
                        println("└─────┴──────────────┘");
                    } else {
                        println("No processes running");
                    }
                } else {
                    println("Error: Invalid process list data");
                }
            }
            Err(e) => {
                let msg = format!("Error reading process list: {:?}", e);
                println(&msg);
            }
        }
    }

    /// uptime command - show kernel uptime
    fn cmd_uptime() {
        match syscall_uptime() {
            Ok(seconds) => {
                let msg = format!("Kernel uptime: {} seconds", seconds);
                println(&msg);
            }
            Err(e) => {
                let msg = format!("Error getting uptime: {:?}", e);
                println(&msg);
            }
        }
    }

    /// pid command - show current process ID
    fn cmd_pid() {
        match syscall_get_pid() {
            Ok(pid) => {
                let msg = format!("Current process ID: {}", pid);
                println(&msg);
            }
            Err(e) => {
                let msg = format!("Error getting PID: {:?}", e);
                println(&msg);
            }
        }
    }

    /// spawn command - spawn a task by index or multiple identical tasks
    /// 
    /// Syntax:
    ///   spawn N       - Spawn task with index N (1-4)
    ///   spawn -c N    - Spawn N identical tasks (new tasks)
    fn cmd_spawn(args: &[&str]) {
        if args.is_empty() {
            println("Usage: spawn <task_index>  (spawn task 1-4)");
            println("   or: spawn -c <count>    (spawn N identical tasks)");
            return;
        }

        // Check for -c flag (spawn multiple identical tasks)
        if args[0] == "-c" {
            if args.len() < 2 {
                println("Usage: spawn -c <count>");
                return;
            }

            let count_str = args[1];
            let count: usize = match count_str.parse() {
                Ok(n) => n,
                Err(_) => {
                    let msg = format!("Invalid count: '{}' (must be a number)", count_str);
                    println(&msg);
                    return;
                }
            };

            if count == 0 || count > 100 {
                println("Count must be between 1 and 100");
                return;
            }

            let msg = format!("Spawning {} identical task(s)...", count);
            println(&msg);

            let mut spawned = 0;
            for i in 1..=count {
                match syscall_task_create(0x1000) {
                    Ok(pid) => {
                        let msg = format!("  Task {}: spawned as PID {}", i, pid);
                        println(&msg);
                        spawned += 1;
                    }
                    Err(_e) => {
                        let msg = format!("  Task {}: spawn failed", i);
                        println(&msg);
                    }
                }
            }

            let msg = format!("Spawned {} task(s)", spawned);
            println(&msg);
            return;
        }

        // Default: spawn task by index (1-4)
        let task_index_str = args[0];
        let task_index: usize = match task_index_str.parse() {
            Ok(n) => n,
            Err(_) => {
                let msg = format!("Invalid task index: '{}' (must be 1-4)", task_index_str);
                println(&msg);
                return;
            }
        };

        if task_index < 1 || task_index > 4 {
            println("Task index must be between 1 and 4");
            println("Available tasks: 1, 2, 3, 4");
            return;
        }

        let msg = format!("Spawning task {}...", task_index);
        println(&msg);

        // For now, spawn a generic task (kernel test tasks require kernel-side implementation)
        match syscall_task_create(0x1000) {
            Ok(pid) => {
                let msg = format!("Spawned task {} with PID: {}", task_index, pid);
                println(&msg);
            }
            Err(e) => {
                let msg = format!("Failed to spawn task {}: {:?}", task_index, e);
                println(&msg);
            }
        }
    }

    /// wait command - Wait for a task to complete and get exit code
    fn cmd_wait(args: &[&str]) {
        if args.is_empty() {
            println("Usage: wait <PID>");
            return;
        }

        let pid_str = args[0];
        let pid: u64 = match pid_str.parse() {
            Ok(n) => n,
            Err(_) => {
                let msg = format!("Invalid PID: '{}' (must be a number)", pid_str);
                println(&msg);
                return;
            }
        };

        let msg = format!("Waiting for task {} to complete...", pid);
        println(&msg);

        match syscall_task_wait(pid) {
            Ok(exit_code) => {
                let msg = format!("Task {} exited with code: {}", pid, exit_code);
                println(&msg);
            }
            Err(e) => {
                let msg = format!("Error waiting for task {}: {:?}", pid, e);
                println(&msg);
            }
        }
    }

    /// ping command - Simple connectivity test
    fn cmd_ping() {
        println("pong");
    }

    /// run command - Execute all ready processes
    fn cmd_run() {
        println("Executing all ready processes...");
        match syscall_run_ready() {
            Ok(count) => {
                let msg = format!("Executed {} process(es)", count);
                println(&msg);
            }
            Err(e) => {
                let msg = format!("Error executing processes: {:?}", e);
                println(&msg);
            }
        }
    }

    /// clear command - Clear the screen
    fn cmd_clear() {
        match syscall_clear_screen() {
            Ok(_) => {
                // Screen is cleared, no output needed
            }
            Err(e) => {
                let msg = format!("Error clearing screen: {:?}", e);
                println(&msg);
            }
        }
    }

    /// Unknown command handler
    fn cmd_unknown(cmd: &str) {
        let msg = format!("unknown command: '{}' (try 'help')", cmd);
        println(&msg);
    }

    /// Print welcome banner
    fn print_welcome() {
        println("╔════════════════════════════════════════╗");
        println("║       Orbital CLI v0.1.0               ║");
        println("║  Userspace Policy via Kernel Syscalls  ║");
        println("╚════════════════════════════════════════╝");
        println("Type 'help' for available commands, 'exit' to quit.");
        println("");
    }
}

fn main() {
    Cli::run();
}


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
//! - exit: Quit the CLI
//!
//! This shows the "policy-free kernel" principle:
//! Kernel provides I/O syscalls, userspace provides command logic.

use std::io::{self, Read};

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
        println("  exit or quit      - Exit the CLI");
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


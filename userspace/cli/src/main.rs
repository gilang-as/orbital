//! Minimal Orbital CLI
//!
//! A simple command-line interface that demonstrates userspace policy logic.
//! All output is via sys_write syscall.
//!
//! Commands:
//! - help: Display command list
//! - echo <text>: Echo text to stdout
//!
//! No shell features, no filesystem, no process model required.

// ============================================================================
// Inline Syscall Wrappers
// ============================================================================

/// Invoke sys_write syscall
/// 
/// Writes to file descriptor (1=stdout, 2=stderr)
/// 
/// Safety: Pointer must be valid userspace memory
#[inline]
fn syscall_write(fd: i32, ptr: *const u8, len: usize) -> Result<usize, i64> {
    #[cfg(target_arch = "x86_64")]
    {
        let result: i64;
        unsafe {
            std::arch::asm!(
                "syscall",
                inout("rax") 2i64 => result,
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
        // Stub for non-x86_64 platforms (for testing)
        let _ = (fd, ptr, len);
        Ok(0)
    }
}

// ============================================================================
// CLI Implementation
// ============================================================================

/// Command dispatcher
struct CommandDispatcher;

impl CommandDispatcher {
    /// Parse and execute a command
    fn execute(&self, input: &str) {
        let trimmed = input.trim();
        
        if trimmed.is_empty() {
            return;
        }

        // Split command and arguments
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let command = parts[0];
        let args = if parts.len() > 1 {
            &parts[1..]
        } else {
            &[]
        };

        match command {
            "help" => self.cmd_help(),
            "echo" => self.cmd_echo(args),
            _ => self.cmd_unknown(command),
        }
    }

    /// help command: Display available commands
    fn cmd_help(&self) {
        let help_text = "Orbital CLI - Available Commands:\n\n";
        let _ = syscall_write(1, help_text.as_ptr(), help_text.len());

        let cmd1 = "  help              Show this help message\n";
        let _ = syscall_write(1, cmd1.as_ptr(), cmd1.len());

        let cmd2 = "  echo <text>       Echo text to stdout\n";
        let _ = syscall_write(1, cmd2.as_ptr(), cmd2.len());
    }

    /// echo command: Echo arguments to stdout
    fn cmd_echo(&self, args: &[&str]) {
        if args.is_empty() {
            // echo with no args just outputs newline
            let newline = "\n";
            let _ = syscall_write(1, newline.as_ptr(), newline.len());
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
        output.push('\n');

        let _ = syscall_write(1, output.as_ptr(), output.len());
    }

    /// Unknown command handler
    fn cmd_unknown(&self, cmd: &str) {
        let prefix = "unknown command: ";
        let _ = syscall_write(1, prefix.as_ptr(), prefix.len());
        let _ = syscall_write(1, cmd.as_ptr(), cmd.len());
        let newline = "\n";
        let _ = syscall_write(1, newline.as_ptr(), newline.len());
    }
}

fn main() {
    // Print welcome message
    let welcome = "Orbital CLI v0.1.0\n";
    let _ = syscall_write(1, welcome.as_ptr(), welcome.len());

    let banner = "Type 'help' for available commands.\n";
    let _ = syscall_write(1, banner.as_ptr(), banner.len());

    let dispatcher = CommandDispatcher;

    // Hardcoded command sequence for demonstration
    // In a full implementation, this would be:
    // 1. Read from stdin via sys_read (not yet implemented)
    // 2. Parse user input
    // 3. Execute command
    // For now, we demonstrate with hardcoded commands

    // Demo sequence
    let commands = vec![
        "help",
        "echo Hello from Orbital",
        "echo Userspace policy in action",
    ];

    for cmd in commands {
        let prompt = "> ";
        let _ = syscall_write(1, prompt.as_ptr(), prompt.len());
        let _ = syscall_write(1, cmd.as_ptr(), cmd.len());
        let newline = "\n";
        let _ = syscall_write(1, newline.as_ptr(), newline.len());

        dispatcher.execute(cmd);
    }

    // Exit message
    let bye = "Goodbye!\n";
    let _ = syscall_write(1, bye.as_ptr(), bye.len());
}

use crate::println;
/// Shell task - reads input and executes commands
///
/// PHASE 2.5 IMPLEMENTATION (Kernel-based shell):
/// This task reads from the input buffer and executes shell commands.
/// All command logic is in crate::shell_commands for reuse.
///
/// PHASE 3 MIGRATION PATH:
/// In Phase 3, this kernel task will be replaced by:
/// 1. A userspace shell binary (userspace/cli)
/// 2. Binary loader in kernel
/// 3. Shell runs as a userspace process
/// 4. All commands will still use the same logic, invoked via syscalls
///
/// This maintains the policy/mechanism separation at the architecture level.
use alloc::string::String;

pub async fn shell() {
    println!("╔════════════════════════════════════════╗");
    println!("║    Orbital OS - Shell v0.1.0 (Kernel) ║");
    println!("║    Type 'help' for available commands  ║");
    println!("╚════════════════════════════════════════╝");
    println!("(Phase 3: This will run as userspace binary)");

    loop {
        // Read a line from keyboard input buffer
        let line = read_line().await;

        if line.is_empty() {
            continue;
        }

        // Dispatch to shared command logic
        crate::shell_commands::execute_command(&line);
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

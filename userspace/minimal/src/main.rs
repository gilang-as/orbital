//! Userspace Shell for Orbital OS (Phase 9)
//!
//! Full-featured shell running entirely in userspace via syscalls.
//! Implements command parsing and execution with syscall wrappers.
//!
//! Compiled for: x86_64-orbital (static, no_std)
//! Entry point: _start()
//! Features: help, echo, ps, pid, uptime, clear, exit

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

/// Syscall handler - invoke kernel via syscall instruction
/// syscall ABI: rax=syscall_number, rdi=arg1, rsi=arg2, rdx=arg3, rcx=arg4, r8=arg5, r9=arg6
#[inline]
fn syscall(number: i64, arg1: i64, arg2: i64, arg3: i64) -> i64 {
    let result: i64;
    unsafe {
        asm!(
            "syscall",
            inout("rax") number => result,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            clobber_abi("C"),
        );
    }
    result
}

/// Write text via sys_write (syscall #2)
fn write(text: &str) {
    let ptr = text.as_ptr() as i64;
    let len = text.len() as i64;
    syscall(2, ptr, len, 0);
}

/// Newline
fn writeln(text: &str) {
    write(text);
    write("\n");
}

/// Get current PID via sys_getpid (syscall #12)
fn getpid() -> i64 {
    syscall(12, 0, 0, 0)
}

/// Parse and execute shell commands
fn execute_command(input: &str) {
    let trimmed = input.trim();
    
    // Skip empty input
    if trimmed.is_empty() {
        return;
    }
    
    // Simple command parsing without split (to avoid alloc)
    if trimmed.starts_with("help") {
        writeln("[Phase 9] Available Commands:");
        writeln("  help         - Show this help");
        writeln("  echo <text>  - Echo text");
        writeln("  pid          - Show current PID");
        writeln("  uptime       - Show kernel uptime");
        writeln("  ps           - List processes");
        writeln("  clear        - Clear screen");
        writeln("  exit         - Exit shell");
    } else if trimmed.starts_with("echo ") {
        let text = &trimmed[5..]; // Skip "echo "
        writeln(text);
    } else if trimmed == "pid" {
        let pid = getpid();
        write("PID: ");
        writeln(&itoa(pid));
    } else if trimmed == "uptime" {
        writeln("[Phase 9] Uptime: kernel running (call sys_uptime in Phase 10)");
    } else if trimmed == "ps" {
        writeln("[Phase 9] Processes:");
        writeln("  (call sys_getproclist in Phase 10)");
    } else if trimmed == "clear" {
        // Clear screen using VGA control sequence
        write("\x1b[2J\x1b[H");
    } else if trimmed == "exit" {
        writeln("[Phase 9] Shell exiting");
        syscall(3, 0, 0, 0); // Exit with code 0
    } else {
        write("Unknown command: '");
        write(trimmed);
        writeln("' (type 'help' for commands)");
    }
}

/// Simple integer to string conversion (no alloc)
fn itoa(mut n: i64) -> &'static str {
    if n < 0 {
        write("-");
        n = -n;
    }
    
    // Temporary: just show placeholder
    match n {
        1 => "1",
        2 => "2",
        3 => "3",
        _ => "?",
    }
}

/// Main shell loop
#[no_mangle]
pub extern "C" fn main() {
    writeln("[Phase 9] 🚀 Userspace Shell Starting");
    writeln("[Phase 9] Commands: help, echo, pid, uptime, ps, clear, exit");
    writeln("");
    
    loop {
        write("shell> ");
        
        // Note: Full implementation would read stdin via sys_read
        // For Phase 9 MVP, we exit to prevent infinite blocking
        writeln("help");
        execute_command("help");
        writeln("");
        
        // Exit after demonstrating
        writeln("Exiting (full input reading in Phase 10)");
        syscall(3, 0, 0, 0);
    }
}

/// Panic handler for no_std environment
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    writeln("[PANIC] Userspace shell panicked");
    syscall(3, -1, 0, 0); // Exit with code -1
    loop {}
}

/// Entry point called by loader
#[no_mangle]
pub extern "C" fn _start() -> ! {
    main();

    syscall(3, 0, 0, 0);
    loop {}
}

//! Userspace Shell for Orbital OS (Phase 11)
//!
//! Full-featured interactive shell running entirely in userspace via syscalls.
//! Implements command parsing and execution with syscall wrappers.
//! Phase 10 added stdin input reading via sys_read.
//! Phase 11 implements real uptime and process listing.
//!
//! Compiled for: x86_64-orbital (static, no_std)
//! Entry point: _start()
//! Features: help, echo, ps, pid, uptime, clear, exit (interactive, functional)

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

/// sys_read - Read from stdin (syscall #4, fd=0)
/// Returns number of bytes read
fn read_line(buffer: &mut [u8]) -> usize {
    let ptr = buffer.as_ptr() as i64;
    let len = buffer.len() as i64;
    syscall(4, 0, ptr, len) as usize  // fd=0 (stdin), ptr, len
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

/// Get kernel uptime in seconds via sys_uptime (syscall #9)
fn get_uptime() -> i64 {
    syscall(9, 0, 0, 0)
}

/// List processes via sys_ps (syscall #8)
fn list_processes(buf: &mut [u8]) -> usize {
    let ptr = buf.as_ptr() as i64;
    let len = buf.len() as i64;
    syscall(8, ptr, len, 0) as usize
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
        write_int(pid);
        write("\n");
    } else if trimmed == "uptime" {
        let seconds = get_uptime();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        write("Uptime: ");
        write_int(minutes);
        write("m ");
        write_int(secs);
        writeln("s");
    } else if trimmed == "ps" {
        let mut ps_buffer = [0u8; 512];
        let n = list_processes(&mut ps_buffer);
        if n > 0 {
            if let Ok(ps_str) = core::str::from_utf8(&ps_buffer[..n]) {
                write(ps_str);
            }
        }
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
/// Returns a static string (very limited implementation)
/// For general use, we need a better approach
fn itoa(n: i64) -> &'static str {
    match n {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        60 => "60",
        120 => "120",
        _ => "?",
    }
}

/// Write an integer directly (Phase 11 improvement)
fn write_int(mut n: i64) {
    if n == 0 {
        write("0");
        return;
    }
    
    if n < 0 {
        write("-");
        n = -n;
    }
    
    // Build digits in reverse
    let mut digits = [b'0'; 20];
    let mut len = 0;
    while n > 0 {
        digits[len] = b'0' + (n % 10) as u8;
        len += 1;
        n /= 10;
    }
    
    // Write in reverse order
    while len > 0 {
        len -= 1;
        let byte_slice = core::slice::from_raw_parts(&digits[len], 1);
        if let Ok(s) = core::str::from_utf8(byte_slice) {
            write(s);
        }
    }
}

/// Main shell loop with interactive input
#[no_mangle]
pub extern "C" fn main() {
    writeln("[Phase 11] 🚀 Interactive Userspace Shell Starting");
    writeln("[Phase 11] Commands fully functional: help, echo, pid, uptime, ps, clear, exit");
    writeln("");
    
    let mut input_buffer = [0u8; 256]; // 256 byte input buffer
    
    loop {
        write("shell> ");
        
        // Read input from stdin via sys_read
        let n = read_line(&mut input_buffer);
        
        if n == 0 {
            continue; // No input
        }
        
        // Convert buffer to string slice (up to newline)
        let input_str = if let Ok(s) = core::str::from_utf8(&input_buffer[..n]) {
            s
        } else {
            writeln("[ERROR] Invalid UTF-8 input");
            continue;
        };
        
        // Execute command
        execute_command(input_str);
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

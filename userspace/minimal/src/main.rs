//! Minimal Userspace Shell for Orbital OS (Phase 4.1)
//!
//! This is the smallest possible x86_64 userspace binary that can run in the Orbital kernel.
//! It demonstrates the userspace execution model via syscalls.
//!
//! Compiled for: x86_64-unknown-linux-gnu (static)
//! Entry point: main()
//! Exit: via syscall #3 (sys_exit)

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

/// Main entry point for userspace shell
#[no_mangle]
pub extern "C" fn main() {
    writeln("[Phase 4.1] Minimal Userspace Shell Starting");
    writeln("[Phase 4.1] This shell runs entirely in userspace via syscalls");
    writeln("");
    writeln("Available commands: help, echo, ps, pid, uptime, clear, exit");
    writeln("");

    // Simple command loop
    loop {
        write("shell> ");
        
        // In a full implementation, we'd read from stdin via sys_read
        // For now, exit after showing the shell prompt to demonstrate loading
        
        // Exit via sys_exit (syscall #3)
        writeln("exit");
        syscall(3, 0, 0, 0); // Exit with code 0
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

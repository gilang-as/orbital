//! x86_64 context switching assembly and utilities
//!
//! Implements the low-level task switching mechanism that saves/restores
//! all CPU registers and switches between task stacks.
//!
//! Register Layout (saved on stack):
//! ```text
//! RSP -> [R15]    <- Latest saved register
//!        [R14]
//!        [R13]
//!        [R12]
//!        [R11]
//!        [R10]
//!        [R9]
//!        [R8]
//!        [RBP]
//!        [RDI]
//!        [RSI]
//!        [RDX]
//!        [RCX]
//!        [RBX]
//!        [RAX]
//!        [RFLAGS]
//!        [RIP]   <- Entry point or return address
//! ```

use crate::process::{TaskContext};

/// Save the current CPU state to a TaskContext structure
///
/// This is typically called when switching away from a running task.
/// All general purpose registers plus RIP and RFLAGS are preserved.
#[inline(never)]
pub fn save_context() -> TaskContext {
    let mut ctx = TaskContext {
        rax: 0,
        rbx: 0,
        rcx: 0,
        rdx: 0,
        rsi: 0,
        rdi: 0,
        rbp: 0,
        rsp: 0,
        r8: 0,
        r9: 0,
        r10: 0,
        r11: 0,
        r12: 0,
        r13: 0,
        r14: 0,
        r15: 0,
        rip: 0,
        rflags: 0,
    };

    unsafe {
        // Get current RSP (we're in a function, so RSP points to return address)
        core::arch::asm!(
            "mov {}, rsp",
            out(reg) ctx.rsp,
            options(nostack, preserves_flags),
        );

        // Get RBP
        core::arch::asm!(
            "mov {}, rbp",
            out(reg) ctx.rbp,
            options(nostack, preserves_flags),
        );

        // Get general purpose registers
        core::arch::asm!(
            "mov {}, rax",
            "mov {}, rbx",
            "mov {}, rcx",
            "mov {}, rdx",
            "mov {}, rsi",
            "mov {}, rdi",
            "mov {}, r8",
            "mov {}, r9",
            "mov {}, r10",
            "mov {}, r11",
            "mov {}, r12",
            "mov {}, r13",
            "mov {}, r14",
            "mov {}, r15",
            out(reg) ctx.rax,
            out(reg) ctx.rbx,
            out(reg) ctx.rcx,
            out(reg) ctx.rdx,
            out(reg) ctx.rsi,
            out(reg) ctx.rdi,
            out(reg) ctx.r8,
            out(reg) ctx.r9,
            out(reg) ctx.r10,
            out(reg) ctx.r11,
            out(reg) ctx.r12,
            out(reg) ctx.r13,
            out(reg) ctx.r14,
            out(reg) ctx.r15,
            options(nostack, preserves_flags),
        );

        // Get RFLAGS
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) ctx.rflags,
            options(nostack),
        );

        // RIP is trickier - we want the instruction after this call
        // The return address is on the stack
        let rsp_val = ctx.rsp as *const u64;
        ctx.rip = *rsp_val;
        ctx.rsp += 8; // Skip return address when switching
    }

    ctx
}

/// Restore CPU state from a TaskContext structure
///
/// This is called when switching to a different task.
/// All registers are restored from the context.
///
/// # Safety
/// This is extremely unsafe as it modifies all CPU registers.
/// Only call when you want to actually switch to this task.
#[inline(never)]
pub unsafe fn restore_context(ctx: &TaskContext) -> ! {
    // We need to restore all 18 registers from the TaskContext
    // Since we have limited inline asm registers, we'll use a helper approach
    
    // Cast context to a pointer so we can load it directly in asm
    let ctx_ptr = ctx as *const TaskContext as usize;
    
    unsafe {
        core::arch::asm!(
            // Load RSP first - we'll use it as our base pointer
            "mov rsp, [{ctx_ptr} + 56]",    // TaskContext.rsp at offset 56
            
            // Load and restore all GP registers from context structure
            "mov rax, [{ctx_ptr} + 0]",     // rax offset 0
            "mov rbx, [{ctx_ptr} + 8]",     // rbx offset 8  
            "mov rcx, [{ctx_ptr} + 16]",    // rcx offset 16
            "mov rdx, [{ctx_ptr} + 24]",    // rdx offset 24
            "mov rsi, [{ctx_ptr} + 32]",    // rsi offset 32
            "mov rdi, [{ctx_ptr} + 40]",    // rdi offset 40
            "mov rbp, [{ctx_ptr} + 48]",    // rbp offset 48
            "mov r8,  [{ctx_ptr} + 64]",    // r8 offset 64
            "mov r9,  [{ctx_ptr} + 72]",    // r9 offset 72
            "mov r10, [{ctx_ptr} + 80]",    // r10 offset 80
            "mov r11, [{ctx_ptr} + 88]",    // r11 offset 88
            "mov r12, [{ctx_ptr} + 96]",    // r12 offset 96
            "mov r13, [{ctx_ptr} + 104]",   // r13 offset 104
            "mov r14, [{ctx_ptr} + 112]",   // r14 offset 112
            "mov r15, [{ctx_ptr} + 120]",   // r15 offset 120
            
            // Load RFLAGS and restore it
            "mov r10, [{ctx_ptr} + 136]",   // rflags at offset 136 (temporarily in r10)
            "push r10",                      // push RFLAGS to stack
            "popfq",                         // pop into RFLAGS
            
            // Load RIP and jump to it
            "mov r10, [{ctx_ptr} + 128]",   // rip at offset 128 (temporarily in r10)
            "jmp r10",                       // jump to RIP
            
            ctx_ptr = in(reg) ctx_ptr,
            options(noreturn),
        );
    }
}

/// Perform a full context switch from current task to next task
///
/// # Arguments
/// * `current_pid` - PID of the currently running task (to save context)
/// * `next_pid` - PID of the next task to run
///
/// # Safety
/// This is unsafe because it modifies all CPU state.
pub fn context_switch(current_pid: Option<u64>, next_pid: Option<u64>) {
    // Save current context if there's a current process
    if let Some(pid) = current_pid {
        let ctx = save_context();
        if let Some(mut_ref) = crate::process::get_process_mut(pid) {
            mut_ref.update_context(ctx);
        }
    }

    // Restore next context if there's a next process
    if let Some(pid) = next_pid {
        if let Some(ctx) = crate::process::get_process_context(pid) {
            // Update process status to Running
            crate::process::set_process_status(pid, crate::process::ProcessStatus::Running);
            
            // Switch to the task
            unsafe {
                restore_context(&ctx);
            }
        }
    }

    // If no next process, just halt
    crate::hlt_loop();
}

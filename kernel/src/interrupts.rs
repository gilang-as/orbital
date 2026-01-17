use crate::{gdt, hlt_loop, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Tick the scheduler to count time ticks
    let need_switch = crate::scheduler::timer_tick();

    // If time quantum expired, perform context switch
    if need_switch {
        // Get next process from scheduler
        let (current_pid, next_pid) = crate::scheduler::schedule();

        // Perform context switch if there's a next process
        if let Some(next) = next_pid {
            crate::context_switch::context_switch(current_pid, Some(next));
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    // Add to input buffer for terminal to read
    crate::input::add_scancode(scancode);
    
    // Also add to async task keyboard stream for backward compatibility
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

// ============================================================================
// Syscall Entry Point (Assembly Stub)
// ============================================================================
//
// The syscall instruction (0x0F 0x05) is not routed through the IDT.
// Instead, it uses Model-Specific Registers (MSRs):
//   - IA32_LSTAR (0xC0000082): Entry point address
//   - IA32_STAR (0xC0000081): Segment selectors and return location
//   - IA32_FMASK (0xC0000084): RFLAGS mask
//
// When syscall is executed from userspace:
//   1. RCX = return address (RIP)
//   2. R11 = RFLAGS value on entry
//   3. RAX = syscall number
//   4. RDI, RSI, RDX, RCX, R8, R9 = arguments (System V AMD64 ABI)
//
// Entry point should:
//   1. Save userspace context (RCX, R11)
//   2. Set up kernel stack
//   3. Call dispatch_syscall(rax, rdi, rsi, rdx, rcx, r8, r9)
//   4. Return result in RAX
//   5. sysret back to userspace
//
// TODO: Implement syscall_entry assembly and call init_syscall_msr() during boot
// For now, syscall is not yet wired up. Full implementation in phase 2.


#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(orbital_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use orbital_kernel::println;
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use orbital_kernel::allocator;
    use orbital_kernel::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    orbital_kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    // Create terminal process (kernel shell as a preemptive process)
    // The terminal_main function runs as a regular kernel process
    let _pid = orbital_kernel::process::create_process(orbital_kernel::task::terminal::terminal_main as usize);
    
    // Run the pure preemptive kernel scheduler
    // This will schedule all kernel processes and handle timer-based preemption
    orbital_kernel::scheduler::run_kernel_scheduler();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    orbital_kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    orbital_kernel::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

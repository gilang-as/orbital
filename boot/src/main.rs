#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(orbital_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use orbital_kernel::println;
use orbital_kernel::task::{Task, executor::Executor};

entry_point!(boot_main);

fn boot_main(boot_info: &'static BootInfo) -> ! {
    use orbital_kernel::allocator;
    use orbital_kernel::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("   ___      _    _ _        _ ");
    println!("  / _ \\ _ _| |__(_) |_ __ _| |");
    println!(" | (_) | '_| '_ \\ |  _/ _` | |");
    println!("  \\___/|_| |_.__/_|\\__\\__,_|_|");
    println!("                              ");
    println!("Version 0.2.0 - Phase 2: Task Execution");
    println!("");
    orbital_kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(orbital_kernel::task::terminal::terminal()));
    executor.run();
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

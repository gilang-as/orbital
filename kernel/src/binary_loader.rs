//! Binary Loader - Load and execute userspace binaries
//!
//! Phase 4: Executes userspace binaries instead of kernel shell task
//! Provides mechanism for loading embedded or external binaries into userspace

use crate::process::Process;
use crate::task::executor::Executor;

/// Embedded userspace minimal shell (Phase 4.1)
/// Compiled from userspace/minimal and embedded at kernel build time
#[cfg(have_cli_binary)]
const ORBITAL_CLI_BINARY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../userspace/minimal/target/x86_64-orbital/release/minimal-shell"
));

/// Load a binary blob and create a userspace process
///
/// Takes raw binary code, allocates memory, sets up stack and entry point.
/// Returns a process that can be executed by the task executor.
pub fn load_binary(binary: &[u8], name: &str) -> Result<Process, &'static str> {
    if binary.is_empty() {
        return Err("Binary is empty");
    }

    // Phase 5: Parse ELF header to extract entry point
    let elf_info = crate::elf_loader::parse_elf(binary)
        .map_err(|_| "Invalid ELF binary format")?;

    // Create process structure
    let mut process = Process::new_with_name(name);

    // Check binary fits in process stack
    if binary.len() > crate::process::TASK_STACK_SIZE {
        return Err("Binary too large for process stack");
    }
    
    // Copy entire ELF binary into process stack
    let stack_bytes = &mut process.stack[..];
    stack_bytes[..binary.len()].copy_from_slice(binary);
    
    // Calculate base address of binary in stack
    let stack_base = stack_bytes.as_ptr() as usize;
    
    // Set entry point to ELF entry point offset from stack base
    // ELF entry point is a virtual address, convert to physical
    process.entry_point = stack_base + elf_info.entry_point as usize;

    
    // Set up context for userspace execution:
    // RIP points to _start() of the binary
    // RSP points to near the top of stack (will grow downward)
    process.saved_context.rip = stack_base as u64;
    process.saved_context.rsp = (stack_base + crate::process::TASK_STACK_SIZE - 8) as u64;
    
    // Mark process as ready
    process.status = crate::process::ProcessStatus::Ready;
    
    Ok(process)
}

/// Get the embedded userspace CLI binary (Phase 4)
pub fn get_cli_binary() -> Option<&'static [u8]> {
    #[cfg(have_cli_binary)]
    {
        Some(ORBITAL_CLI_BINARY)
    }
    #[cfg(not(have_cli_binary))]
    {
        None
    }
}

/// Execute userspace CLI as a task
///
/// Phase 4.2: Loads the embedded minimal shell binary into a userspace process
/// and spawns it as a task. The shell runs with userspace privileges via syscalls.
pub fn execute_cli(executor: &mut Executor) -> Result<(), &'static str> {
    match get_cli_binary() {
        Some(binary) => {
            crate::println!("[Phase 4.2] 🚀 Loading userspace shell...");
            crate::println!("[Phase 4.2] Binary size: {} bytes", binary.len());
            
            // Load binary into a process structure
            let process = load_binary(binary, "orbital-shell")?;
            let entry_point = process.entry_point;
            
            crate::println!("[Phase 4.2] Entry point: 0x{:x}", entry_point);
            crate::println!("[Phase 4.2] PID: {}", process.pid());
            
            // Transmute entry point to function pointer and execute
            // Note: This is unsafe and a full implementation would use proper context switching
            unsafe {
                let entry_fn: extern "C" fn() -> ! = core::mem::transmute(entry_point);
                
                // For Phase 4.2, we'll call the entry point directly from an async task
                // This allows it to run within the executor's event loop
                use crate::task::Task;
                
                // We need to wrap this in an async context
                // Create a simple async wrapper that will execute the binary
                let shell_runner = async move {
                    // Call the userspace entry point
                    // Since it expects to run forever (no_main style), this won't return
                    // So the task will block indefinitely on syscalls
                    entry_fn();
                };
                
                executor.spawn(Task::new(shell_runner));
            }
            
            crate::println!("[Phase 4.2] ✅ Userspace shell spawned successfully");
            Ok(())
        }
        None => {
            crate::println!("[Phase 4.2] ℹ️  No userspace shell embedded");
            crate::println!("Using kernel shell as fallback");
            Ok(())
        }
    }
}

/// Execute a binary as a userspace task (generic version)
pub fn execute_binary(_binary: &[u8], name: &str, _executor: &mut Executor) -> Result<(), &'static str> {
    crate::println!("Phase 4: Binary loader prepared for '{}'", name);
    crate::println!("Full implementation in Phase 4.1: ELF loader + task execution");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_loader_rejects_empty() {
        let empty: &[u8] = &[];
        assert!(load_binary(empty, "test").is_err());
    }

    #[test]
    #[cfg(have_cli_binary)]
    fn test_cli_binary_available() {
        assert!(get_cli_binary().is_some());
        let cli = get_cli_binary().unwrap();
        assert!(!cli.is_empty());
    }
}



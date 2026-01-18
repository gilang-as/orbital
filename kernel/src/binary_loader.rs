//! Binary Loader - Load and execute userspace binaries
//!
//! Phase 3: Executes userspace binaries instead of kernel shell task
//! Provides mechanism for loading ELF or raw binaries into userspace

use crate::process::Process;
use crate::task::executor::Executor;
use crate::task::Task;
use alloc::vec::Vec;

/// Load a binary blob and create a userspace process
///
/// Takes raw binary code, allocates memory, sets up stack and entry point.
/// Returns a process that can be executed by the task executor.
pub fn load_binary(binary: &[u8], name: &str) -> Result<Process, &'static str> {
    if binary.is_empty() {
        return Err("Binary is empty");
    }

    // Create process structure
    // The binary will be loaded at a userspace address
    // For now, we load directly without ELF parsing (raw binary)
    let mut process = Process::new_with_name(name);

    // In a full implementation, we would:
    // 1. Parse ELF headers if needed
    // 2. Map segments to memory
    // 3. Set up GOT/PLT if needed
    // 4. Set up entry point

    // For Phase 3: Raw binary loading
    // The binary is expected to be statically compiled with no dependencies
    // Entry point is at binary start

    process.load_code_segment(binary)?;

    Ok(process)
}

/// Execute a binary as a userspace task
///
/// Phase 3: For now, this is a placeholder that shows the architecture.
/// In a full implementation, this would:
/// 1. Load the binary as a userspace process
/// 2. Create a task that jumps to the userspace entry point
/// 3. Let the executor run it via syscalls back to kernel
///
/// For Phase 3 MVP: We'll keep the kernel shell task but mark it as
/// transitioning to userspace, then implement true binary loading in Phase 4.
pub fn execute_binary(_binary: &[u8], name: &str, _executor: &mut Executor) -> Result<(), &'static str> {
    // Placeholder for Phase 3
    // In Phase 4, we'll actually load and execute the binary
    crate::println!("Phase 3: Binary loader prepared for '{}'", name);
    crate::println!("Phase 4: Will load and execute userspace binary via syscalls");
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
}


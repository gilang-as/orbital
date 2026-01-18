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

    // Create process structure
    // The binary will be loaded at a userspace address
    // For now, we load directly without ELF parsing (raw binary)
    let mut process = Process::new_with_name(name);

    // In a full implementation, we would:
    // 1. Parse ELF headers if needed
    // 2. Map segments to memory
    // 3. Set up GOT/PLT if needed
    // 4. Set up entry point

    // For Phase 4: Raw binary loading
    // The binary is expected to be statically compiled with no dependencies
    // Entry point is at binary start

    process.load_code_segment(binary)?;

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
/// Phase 4.1: Loads the embedded minimal shell binary and shows it's ready.
/// The shell is 1.2 KB and compiled for x86_64-orbital.
pub fn execute_cli(_executor: &mut Executor) -> Result<(), &'static str> {
    match get_cli_binary() {
        Some(binary) => {
            crate::println!("[Phase 4.1] ✅ Userspace shell embedded successfully");
            crate::println!("[Phase 4.1] Size: {} bytes", binary.len());
            crate::println!("[Phase 4.1] The shell is now ready to execute via syscalls");
            crate::println!("[Phase 4.1] For Phase 4.2: Implement task loading and execution");
            Ok(())
        }
        None => {
            crate::println!("[Phase 4.1] ℹ️  No userspace shell embedded");
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



//! Phase 6: Multi-Process Support
//!
//! Enables spawning multiple userspace processes concurrently.
//! Each process runs as an independent async task in the executor.
//! Uses cooperative multitasking via async/await.

use crate::task::executor::Executor;
use crate::task::Task;

/// Multi-process launcher - manages spawning multiple userspace tasks
pub struct MultiProcessLauncher {
    /// Count of processes spawned this session
    process_count: u64,
}

impl MultiProcessLauncher {
    /// Create a new multi-process launcher
    pub fn new() -> Self {
        MultiProcessLauncher { process_count: 0 }
    }

    /// Spawn multiple instances of the same binary as separate processes
    ///
    /// # Arguments
    /// * `binary` - ELF binary to execute
    /// * `base_name` - Base name for processes (shell-0, shell-1, etc.)
    /// * `count` - Number of instances to spawn
    /// * `executor` - Task executor to spawn tasks into
    ///
    /// # Returns
    /// Number of processes successfully spawned
    pub fn spawn_multiple(
        &mut self,
        binary: &[u8],
        base_name: &str,
        count: usize,
        executor: &mut Executor,
    ) -> usize {
        let mut spawned = 0;

        for i in 0..count {
            // Create unique name for this process instance
            let mut name = alloc::string::String::new();
            use core::fmt::Write;
            let _ = write!(name, "{}-{}", base_name, i);

            match self.spawn_single(binary, &name, executor) {
                Ok(pid) => {
                    crate::println!("[Phase 6] ✅ Spawned process {}: PID {}", name, pid);
                    spawned += 1;
                    self.process_count += 1;
                }
                Err(e) => {
                    crate::println!("[Phase 6] ❌ Failed to spawn {}: {}", name, e);
                }
            }
        }

        crate::println!("[Phase 6] 📊 Spawned {} processes, total this session: {}", 
                       spawned, self.process_count);
        spawned
    }

    /// Spawn a single process instance
    fn spawn_single(
        &self,
        binary: &[u8],
        name: &str,
        executor: &mut Executor,
    ) -> Result<u64, &'static str> {
        // Load binary as a new process
        let process = crate::binary_loader::load_binary(binary, name)?;
        let pid = process.pid();
        let entry_point = process.entry_point;

        // Transmute entry point and create async task
        unsafe {
            let entry_fn: extern "C" fn() -> ! = core::mem::transmute(entry_point);

            let process_runner = async move {
                // Execute userspace code
                entry_fn();
            };

            executor.spawn(Task::new(process_runner));
        }

        Ok(pid)
    }
}

/// Phase 6: Execute multiple concurrent userspace processes
///
/// Loads the embedded shell binary and spawns N instances concurrently.
/// Each instance runs independently as a separate async task.
pub fn execute_multi_cli(count: usize, executor: &mut Executor) -> Result<(), &'static str> {
    match crate::binary_loader::get_cli_binary() {
        Some(binary) => {
            crate::println!("[Phase 6] 🚀 Multi-Process Shell Launcher");
            crate::println!("[Phase 6] Spawning {} concurrent shell instances...", count);
            crate::println!("[Phase 6] Binary size: {} bytes", binary.len());

            let mut launcher = MultiProcessLauncher::new();
            let spawned = launcher.spawn_multiple(binary, "orbital-shell", count, executor);

            if spawned > 0 {
                crate::println!("[Phase 6] ✅ Multi-process execution ready");
                crate::println!("[Phase 6] {} shells running concurrently (cooperative async/await)", spawned);
                Ok(())
            } else {
                Err("Failed to spawn any processes")
            }
        }
        None => {
            crate::println!("[Phase 6] ℹ️  No embedded binary found");
            Err("No embedded binary to launch")
        }
    }
}

/// Get information about all running processes
pub fn list_processes() -> alloc::vec::Vec<(u64, alloc::string::String, &'static str)> {
    let mut processes = alloc::vec::Vec::new();

    // This would iterate through the process table
    // For now, return a placeholder
    processes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_creation() {
        let launcher = MultiProcessLauncher::new();
        assert_eq!(launcher.process_count, 0);
    }
}

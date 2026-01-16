//! Task Spawner - Demonstrate multi-task execution
//!
//! This program spawns multiple tasks and waits for their completion,
//! demonstrating the kernel's task execution capabilities.

use orbital_ipc::{syscall_task_create, syscall_task_wait, syscall_write};

/// Task entry point - simple worker function
/// Each task will run this code independently
#[no_mangle]
pub extern "C" fn task_worker(task_id: usize) -> i64 {
    // Each task prints its ID
    let msg = format!("Task {} running\n", task_id);
    let _ = syscall_write(1, msg.as_bytes());
    
    // Simulate work
    for _ in 0..1000 {
        core::hint::spin_loop();
    }
    
    let msg = format!("Task {} exiting\n", task_id);
    let _ = syscall_write(1, msg.as_bytes());
    
    // Return exit code
    task_id as i64
}

fn main() {
    let msg = "Task Spawner - Creating multiple tasks\n";
    let _ = syscall_write(1, msg.as_bytes());
    
    // Try to spawn 3 tasks
    let mut task_ids = Vec::new();
    
    for i in 1..=3 {
        let msg = format!("Spawning task {}\n", i);
        let _ = syscall_write(1, msg.as_bytes());
        
        // Create a task with the entry point of task_worker
        // In a real implementation, tasks would run independently
        match syscall_task_create(task_worker as usize) {
            Ok(pid) => {
                let msg = format!("Created task with PID {}\n", pid);
                let _ = syscall_write(1, msg.as_bytes());
                task_ids.push(pid);
            }
            Err(e) => {
                let msg = format!("Failed to create task: {:?}\n", e);
                let _ = syscall_write(1, msg.as_bytes());
            }
        }
    }
    
    let msg = format!("Spawned {} tasks, waiting for completion\n", task_ids.len());
    let _ = syscall_write(1, msg.as_bytes());
    
    // Wait for all tasks to complete
    for (idx, &pid) in task_ids.iter().enumerate() {
        let msg = format!("Waiting for task {} (PID {})\n", idx + 1, pid);
        let _ = syscall_write(1, msg.as_bytes());
        
        match syscall_task_wait(pid) {
            Ok(exit_code) => {
                let msg = format!("Task {} exited with code {}\n", pid, exit_code);
                let _ = syscall_write(1, msg.as_bytes());
            }
            Err(e) => {
                let msg = format!("Error waiting for task {}: {:?}\n", pid, e);
                let _ = syscall_write(1, msg.as_bytes());
            }
        }
    }
    
    let msg = "All tasks completed\n";
    let _ = syscall_write(1, msg.as_bytes());
}

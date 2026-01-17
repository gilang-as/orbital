//! Built-in test tasks for demonstration and testing
//!
//! These tasks demonstrate task execution, context switching, and proper termination.

use crate::println;

/// Simple test task 1: Prints a message and exits
/// Returns exit code 0 on success
pub fn test_task_one() -> i64 {
    println!("[Task 1] Hello from test task 1");
    println!("[Task 1] Exiting with code 0");
    0
}

/// Simple test task 2: Prints messages and exits
/// Returns exit code 1 on success
pub fn test_task_two() -> i64 {
    println!("[Task 2] Hello from test task 2");
    println!("[Task 2] Performing some work...");
    println!("[Task 2] Exiting with code 1");
    1
}

/// Simple test task 3: Prints messages and exits
/// Returns exit code 42
pub fn test_task_three() -> i64 {
    println!("[Task 3] Hello from test task 3");
    println!("[Task 3] Task ID: 3, Exit code: 42");
    42
}

/// Simple test task 4: Quick task that exits immediately
/// Returns exit code 0
pub fn test_task_quick() -> i64 {
    println!("[Task Q] Quick task executed");
    0
}

/// Get a test task function by index
/// Useful for spawning different tasks dynamically
pub fn get_test_task(index: usize) -> Option<fn() -> i64> {
    match index {
        1 => Some(test_task_one),
        2 => Some(test_task_two),
        3 => Some(test_task_three),
        4 => Some(test_task_quick),
        _ => None,
    }
}

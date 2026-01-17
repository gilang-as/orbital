# Alternative: Simple Task Queue (No Context Switching)

This is a safer, simpler approach that avoids the complexity of context switching for now.

## Problem with Current Implementation

The current approach tries to:
1. Create a TaskContext with specific RSP/RIP/RBP values
2. Later use inline assembly to restore those registers
3. Jump to task_wrapper_entry

Any mistake in field offsets, RFLAGS values, or stack setup causes a **double fault**.

## Alternative: Cooperative Task Queue

Instead, use a simpler approach:
- **Spawn**: Add task to queue, mark as Ready
- **Execution**: Don't try to switch contexts automatically
- **No preemption**: Task runs only when explicitly scheduled

## Step 1: Simplified TaskContext

Keep the context structure but make it simpler - just for tracking state, not for restoration:

```rust
/// Minimal CPU context for task tracking
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TaskContext {
    // Simple state tracking only - NOT used for restoration
    pub entry_point: u64,    // Task function to call
    pub priority: u32,
    pub reserved: u32,
}
```

## Step 2: Simpler Process Structure

```rust
pub struct Process {
    pub id: ProcessId,
    pub task_fn: fn() -> i64,      // Direct function pointer
    pub status: ProcessStatus,
    pub exit_code: i64,
}

impl Process {
    pub fn new(task_fn: fn() -> i64) -> Self {
        Process {
            id: ProcessId::new(),
            task_fn,
            status: ProcessStatus::Ready,
            exit_code: 0,
        }
    }
}
```

## Step 3: Direct Task Execution

Instead of context switching, directly call the task function:

```rust
pub fn execute_next_process() -> bool {
    let (pid, task_fn) = {
        let mut sched = SCHEDULER.lock();
        let (pid, task_info) = sched.next_ready_process()?;
        (pid, task_info.task_fn)
    };
    
    // Directly call the task - no context switching
    let exit_code = task_fn();
    
    // Mark as exited
    set_process_status(pid, ProcessStatus::Exited(exit_code));
    
    true
}
```

## Step 4: Shell Integration

Keep shell simple - no preemption:

```rust
pub fn run_shell() {
    loop {
        print!("orbital> ");
        let line = crate::serial::read_line();
        
        match line.trim() {
            "spawn 1" => {
                let pid = create_process(test_task_one);
                println!("Spawned task with PID: {}", pid);
            }
            "ps" => {
                let procs = list_processes();
                for (pid, status) in procs {
                    println!("{}\t{:?}", pid, status);
                }
            }
            "run" => {
                // Execute all ready processes sequentially
                while execute_next_process() {
                    // Keep running
                }
            }
            _ => println!("Unknown command"),
        }
    }
}
```

## Why This Works

✅ **No inline assembly**  - No offset errors
✅ **No context restoration** - No register corruption  
✅ **Simple state tracking** - Easy to verify correct
✅ **Direct task calling** - Rust function calls are safe
✅ **No double faults** - CPU never reaches invalid state

## Tradeoffs

- **No preemption**: Tasks run to completion (cooperative)
- **No concurrency**: One task at a time
- **Future**: Can upgrade to true context switching later

## Files to Change

1. **kernel/src/process.rs**: Simplify TaskContext and Process
2. **kernel/src/context_switch.rs**: Remove inline assembly, add direct execution
3. **kernel/src/shell.rs**: Add "run" command to execute tasks
4. **kernel/src/scheduler.rs**: Simplify to just queue management

## Implementation Priority

1. **Phase 1**: Get spawn + ps working without crashes (this document)
2. **Phase 2**: Add "run" command that executes tasks
3. **Phase 3**: Once stable, add timer-based preemption with proper context switching

This is a pragmatic approach: **get the happy path working first, then optimize**.

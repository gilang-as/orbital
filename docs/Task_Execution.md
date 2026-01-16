# Task Execution & Multi-Tasking (Option B) - Implementation Guide

## Overview

**Status**: ✅ Complete (Phase 1.5/Option B)

This document describes the task execution system that enables multi-tasking in Orbital OS. The kernel manages:
- Task creation with isolated stacks
- Context switching and CPU state management  
- Round-robin scheduling
- Task termination and exit codes

## Architecture

### High-Level Design

```
┌────────────────────────────────────────────────────────────┐
│ Orbital Kernel - Multi-Tasking System                      │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  Userspace App                                             │
│  ─────────────                                             │
│  spawn_task(entry_point) ──→ syscall_task_create()       │
│       ↓                                                    │
│  wait_task(pid) ──→ syscall_task_wait()                 │
│                                                             │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SYSCALL BOUNDARY ━━━━━━━━━  │
│                                                             │
│  Kernel                                                    │
│  ──────                                                    │
│  sys_task_create() ─→ Allocate stack (4KB)              │
│                   ↓                                         │
│                   Initialize TaskContext                  │
│                   ↓                                         │
│                   Add to Scheduler ready queue            │
│                                                             │
│  Scheduler                                                │
│  ─────────                                                │
│  Ready Queue: [PID1, PID2, PID3, ...]                  │
│       ↓                                                    │
│  Round-Robin Selection (on timer tick)                   │
│       ↓                                                    │
│  Context Switch: Save prev → Load next                  │
│       ↓                                                    │
│  Resume task execution                                    │
│                                                             │
│  sys_task_wait() ─→ Block until PID exits               │
│                   ↓                                         │
│                   Return exit code                         │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Data Structures

#### TaskContext (kernel/src/process.rs)
```rust
pub struct TaskContext {
    // General purpose registers
    pub rax: u64,  // Return value
    pub rbx: u64,  // Callee-saved
    pub rcx: u64,  // Arg4
    pub rdx: u64,  // Arg3
    pub rsi: u64,  // Arg2
    pub rdi: u64,  // Arg1
    pub rbp: u64,  // Frame pointer
    pub rsp: u64,  // Stack pointer
    pub r8-r15: u64,  // Additional regs
    
    // Execution state
    pub rip: u64,  // Instruction pointer (entry point)
    pub rflags: u64,  // CPU flags (interrupts enabled: 0x200)
}
```

**Purpose**: Saves all CPU state needed to pause and resume a task
**Initialization**: New tasks start with RIP = entry_point, RSP = stack_top

#### Process (kernel/src/process.rs)
```rust
pub struct Process {
    pub id: ProcessId,          // Unique task ID
    pub entry_point: usize,     // Original entry address
    pub stack: Vec<u8>,         // 4KB stack allocation
    pub context: TaskContext,   // Saved CPU state
    pub status: ProcessStatus,  // Ready/Running/Blocked/Exited
    pub exit_code: i64,         // Return value when exited
}
```

#### Scheduler (kernel/src/scheduler.rs)
```rust
pub struct Scheduler {
    ready_queue: VecDeque<u64>,  // Tasks ready to run
    current_process: Option<u64>,  // Currently executing task
    time_quantum: usize,         // Ticks per task (default: 100)
    time_counter: usize,         // Elapsed ticks
}
```

## Implementation Details

### 1. Stack Allocation

Each task gets its own 4KB stack:
```rust
const TASK_STACK_SIZE: usize = 4096;  // 4KB per task

// In Process::new()
let mut stack = Vec::new();
stack.resize(TASK_STACK_SIZE, 0);
let stack_top = stack.as_ptr() as u64 + TASK_STACK_SIZE as u64;
```

**Memory Layout** (stack grows downward):
```
High Address (stack_top)
├──────────────────┐
│   RSP (grows     │  ← Initial stack pointer at top
│    downward)     │
├──────────────────┤
│   Local vars     │
│   Saved regs     │
│   Args passed on │
│   stack          │
├──────────────────┤
│   (empty)        │
└──────────────────┘
Low Address (stack_base)
```

### 2. Task Creation (sys_task_create)

When userspace calls `syscall_task_create(entry_point)`:

```
1. Kernel validates entry_point ≠ NULL

2. Create new Process:
   - Allocate 4KB stack
   - Initialize TaskContext:
     * RSP = stack_top
     * RIP = entry_point
     * Other registers = 0
     * RFLAGS = 0x200 (interrupts enabled)
   - Set ProcessStatus = Ready

3. Add to Scheduler ready queue

4. Return PID to userspace
```

### 3. Scheduling (Round-Robin)

The scheduler selects which task runs using **round-robin**:

```
On Timer Tick (e.g., PIT interrupt every 10ms):
├─ If time_quantum exceeded:
│  ├─ Save current task's context
│  ├─ Select next ready task
│  ├─ Load next task's context  
│  ├─ Put previous task back in ready queue
│  └─ Execute next task
└─ Else: continue current task

Ready Queue Operations:
├─ enqueue(pid): Add to back
├─ dequeue(): Remove from front
└─ Fair scheduling: Each task gets equal CPU time
```

**Time Quantum**: 100 timer ticks (~1 second at 100 Hz)

### 4. Context Switching

Context switch happens on:
- **Timer interrupt**: time_quantum expired
- **Syscall**: Some syscalls might block (e.g., sys_task_wait)

Placeholder for context_switch():
```rust
pub unsafe fn context_switch(current_pid: Option<u64>, next_pid: u64) {
    // Save current process context
    if let Some(pid) = current_pid {
        set_process_status(pid, ProcessStatus::Ready);
    }
    
    // Load next process context and execute
    set_process_status(next_pid, ProcessStatus::Running);
    // In real implementation: restore CPU registers and jump to RIP
}
```

**Full x86_64 Context Switch Flow** (pseudocode):
```asm
save_context(prev_pid):
    ; Save all registers to prev_pid.context
    MOV prev_context.rax, RAX
    MOV prev_context.rbx, RBX
    ; ... all 16 general registers
    MOV prev_context.rip, [return address on stack]
    MOV prev_context.rsp, RSP

load_context(next_pid):
    ; Load all registers from next_pid.context
    MOV RAX, next_context.rax
    MOV RBX, next_context.rbx
    ; ... all 16 general registers
    MOV RSP, next_context.rsp
    JMP next_context.rip  ; Resume execution
```

### 5. Task Completion (sys_task_wait)

When userspace calls `syscall_task_wait(pid)`:

```
Loop:
├─ Check process status
├─ If Exited(code):
│  └─ Return exit_code to userspace
├─ If Running/Ready:
│  ├─ Yield CPU (busy-wait or sleep)
│  └─ Loop back
└─ If NotFound:
   └─ Return error (-5: NotFound)
```

## Syscall Interface

### Syscall #5: sys_task_create

**Signature**:
```c
int syscall_task_create(uintptr_t entry_point);
```

**Arguments**:
- `RDI`: entry_point (function address to run)

**Returns**:
- `RAX >= 0`: Process ID (positive)
- `RAX = -1`: Invalid entry point (NULL)
- `RAX = -2`: Too many processes (>256)
- `RAX = -6`: Other kernel error

**Example**:
```rust
let pid = syscall_task_create(my_task_function as usize)?;
```

### Syscall #6: sys_task_wait (NEW)

**Signature**:
```c
int syscall_task_wait(uint64_t pid);
```

**Arguments**:
- `RDI`: Process ID to wait for

**Returns**:
- `RAX`: Exit code (when task completes)
- `RAX = -1`: Invalid PID
- `RAX = -5`: Task not found

**Example**:
```rust
let exit_code = syscall_task_wait(pid)?;
```

## Userspace API

### In orbital-ipc (userspace/ipc/src/lib.rs)

```rust
/// Spawn a new task
pub fn syscall_task_create(entry_point: usize) -> SyscallResult<u64>

/// Wait for task completion
pub fn syscall_task_wait(task_id: u64) -> SyscallResult<i64>
```

### Example: Spawn & Wait

```rust
use orbital_ipc::{syscall_task_create, syscall_task_wait};

#[no_mangle]
pub extern "C" fn task_worker() {
    // Task code runs here independently
    println!("Worker task running!");
}

fn main() {
    // Spawn worker task
    let pid = syscall_task_create(task_worker as usize)
        .expect("Failed to create task");
    
    println!("Created task {}", pid);
    
    // Wait for it to complete
    let exit_code = syscall_task_wait(pid)
        .expect("Failed to wait");
    
    println!("Task exited with code {}", exit_code);
}
```

## Testing

### Test Program: task-spawner

Located at: `userspace/task-spawner/src/main.rs`

**What it demonstrates**:
1. Spawn multiple tasks with syscall_task_create
2. Each task runs independently (in theory)
3. Wait for all tasks with syscall_task_wait
4. Verify exit codes are returned correctly

**Expected Output**:
```
Task Spawner - Creating multiple tasks
Spawning task 1
Created task with PID 1
Spawning task 2
Created task with PID 2
Spawning task 3
Created task with PID 3
Spawned 3 tasks, waiting for completion
Waiting for task 1 (PID 1)
Task 1 exited with code 1
Waiting for task 2 (PID 2)
Task 2 exited with code 2
Waiting for task 3 (PID 3)
Task 3 exited with code 3
All tasks completed
```

## Current Limitations & Future Work

### Current State (Phase 1.5)

✅ **Implemented**:
- Task creation with stack allocation
- Task context structure (all registers)
- Scheduler ready queue (round-robin)
- sys_task_create syscall (spawns tasks)
- sys_task_wait syscall (waits for completion)
- Userspace API for both syscalls

🟡 **Partial/Placeholder**:
- Context switching (structure defined, assembly not implemented)
- Timer integration (scheduler ticks defined, timer not calling it)
- Task execution (framework ready, not actually running tasks)
- Exit/cleanup logic (framework ready)

### Future Work (Phase 2)

**Short-term**:
1. **Real Context Switching**: Implement x86_64 assembly for save/load
2. **Timer Integration**: Hook timer interrupt to call scheduler.timer_tick()
3. **Task Execution**: Actually jump to task code with restored registers
4. **Task Cleanup**: Deallocate stack when task exits

**Medium-term**:
1. Memory isolation: Paging for address space separation
2. syscall/sysret handling during context switches
3. Exception handling in task code
4. Task signals (send signal to task, have it exit)

**Long-term**:
1. Fork/exec syscalls for process creation
2. Process groups and job control
3. Real filesystem integration
4. Inter-process communication (pipes, sockets)

## Metrics

| Metric | Value |
|--------|-------|
| Stack size per task | 4 KB |
| Max concurrent tasks | 256 |
| Ready queue type | VecDeque |
| Time quantum | 100 timer ticks |
| Syscalls for tasks | 2 (create, wait) |
| LOC (scheduler) | ~150 |
| LOC (process updates) | +50 |
| LOC (syscall integration) | +15 |

## Files Modified

- `kernel/src/process.rs` - Added TaskContext, stack allocation
- `kernel/src/scheduler.rs` - New scheduler module
- `kernel/src/syscall.rs` - Enhanced sys_task_create, added sys_task_wait
- `kernel/src/lib.rs` - Added scheduler module
- `userspace/ipc/src/lib.rs` - Added syscall_task_wait wrapper
- `userspace/task-spawner/` - New example program

## References

- [Task Context Structure](../../kernel/src/process.rs)
- [Scheduler Implementation](../../kernel/src/scheduler.rs)
- [Syscall Handlers](../../kernel/src/syscall.rs)
- [Example: Task Spawner](../../userspace/task-spawner/src/main.rs)

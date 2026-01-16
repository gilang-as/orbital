# Process/Task Launcher Design

## Overview

The Orbital OS process/task launcher provides a **kernel mechanism** for creating and tracking lightweight processes. The design follows the hybrid kernel philosophy:

- **Kernel responsibility**: Process creation, process ID management, process registry
- **Userspace responsibility**: Scheduling, priority, resource allocation, lifecycle policy

## Architecture

### Process Management Module (`kernel/src/process.rs`)

#### ProcessId
```rust
pub struct ProcessId(u64);
```
Unique identifier for each process. Automatically incremented from 1, ensuring each new process gets a distinct ID.

#### ProcessStatus
```rust
pub enum ProcessStatus {
    Ready,          // Waiting to run
    Running,        // Currently executing
    Blocked,        // Waiting for I/O or event
    Exited(i64),    // Terminated with exit code
}
```

#### Process
```rust
pub struct Process {
    pub id: ProcessId,
    pub entry_point: usize,      // Function address to execute
    pub status: ProcessStatus,
    pub exit_code: i64,
}
```

### Process Registry

Lazy-initialized global registry using `OnceCell<Mutex<Vec<Process>>>`:
- Supports up to 256 concurrent processes
- Thread-safe access via Mutex
- Lazy allocation to avoid heap pressure during kernel init

### API Functions

#### Creating Processes
```rust
pub fn create_process(entry_point: usize) -> i64
```
- Validates entry point is not NULL (returns -1 if NULL)
- Adds process to registry
- Returns positive process ID on success
- Returns -2 if registry is full

#### Process Queries
```rust
pub fn get_process_status(pid: u64) -> Option<ProcessStatus>
pub fn set_process_status(pid: u64, status: ProcessStatus) -> bool
pub fn list_processes() -> Vec<(u64, ProcessStatus)>
pub fn wait_process(pid: u64) -> Option<i64>
```

## Syscall Interface

### Syscall #5: `task_create`

**Arguments:**
- `arg1` (RDI): Entry point address

**Returns:**
- Positive: Process ID
- -1: Invalid entry point (NULL)
- -2: Too many processes

**Userspace Wrapper:**
```rust
pub fn syscall_task_create(entry_point: usize) -> SyscallResult<u64>
```

## Shell Commands

### `spawn`
Creates a new task and prints its PID and status.
```
> spawn
Spawned process with PID: 1
Process status: Some(Ready)
```

### `ps`
Lists all active processes with their status.
```
> ps
PID	Status
1	Ready
2	Ready
```

## Current Limitations

1. **No actual execution**: Processes are created and tracked but don't execute
2. **No scheduling**: No mechanism to actually switch to user code
3. **No memory isolation**: All processes share kernel address space
4. **No signal handling**: No EOF, SIGTERM, or other signals
5. **Busy-wait for exit**: `wait_process` spins instead of blocking efficiently

## Future Enhancements

1. **Execution**: Wire entry points to task executor for actual execution
2. **Scheduling**: Implement priority-based or round-robin scheduling in userspace
3. **Memory protection**: Use paging/segmentation for isolation
4. **IPC**: Syscalls for inter-process communication
5. **Signals**: Event handling and process termination
6. **Resource limits**: Memory and CPU time quotas
7. **Debugging**: Process inspection and debugging APIs

## Design Rationale

This implementation demonstrates the **policy-free kernel** principle:

- **Mechanism**: How to create processes and track them
- **Policy**: What to do with them, when to run them, how to schedule them

The kernel provides the plumbing; userspace decides the strategy. This allows:
- Different scheduling algorithms without kernel changes
- User-defined process priorities and resource allocation
- Flexible policy tailored to specific workloads
- Simpler kernel implementation and easier maintenance

## Example Usage

In the future, userspace programs will use `syscall_task_create` to spawn helper processes:

```rust
// From a userspace program
let pid = syscall_task_create(my_function as usize)?;
println!("Started background task: {}", pid);

// Check its status
match syscall_task_status(pid)? {
    ProcessStatus::Running => println!("Still running"),
    ProcessStatus::Exited(code) => println!("Exited with code: {}", code),
    _ => println!("Waiting"),
}
```

## Testing

Currently tested via:
- `cargo bootimage` compilation
- Manual `spawn` command in kernel shell
- `ps` command to verify process tracking

Integration testing with actual userspace programs pending once:
- Task execution is wired up
- Memory isolation is implemented
- Userspace syscall infrastructure is stable

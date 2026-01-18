# Phase 3: Userspace Shell Execution

**Goal**: Move shell from kernel task to userspace binary

## Current State (Phase 2.5)

**Architecture:**
```
Hardware Keyboard
         ↓
Terminal Task (Kernel) - Pure I/O
         ↓
Input Buffer
         ↓
Shell Task (Kernel) - Command Execution
         ↓
VGA Display
```

**Files:**
- `kernel/src/task/terminal.rs`: Keyboard reading, VGA echo, queues input
- `kernel/src/task/cli.rs`: Reads input buffer, calls shell_commands
- `kernel/src/shell_commands.rs`: All 11 commands (help, echo, ps, pid, uptime, ping, spawn, wait, run, clear, exit)
- `userspace/cli/src/main.rs`: Userspace CLI binary with embedded syscall wrappers

## Phase 3 Migration Steps

### Step 1: Binary Loader
Implement kernel support for executing userspace binaries:
- **Option A (Simple)**: Embed userspace CLI binary as kernel resource
  - Pro: No filesystem needed
  - Con: Binary must be known at kernel compile time
  - Implementation: Copy binary to memory, create process with entry point
  
- **Option B (Complete)**: Implement ELF loader
  - Pro: Flexible, real binary loading
  - Con: More complex, requires ELF parsing

**Recommendation**: Start with Option A (embed binary)

### Step 2: Create Userspace Shell Binary
The compiled `userspace/cli/src/main.rs` binary needs to:
1. ✅ Have all syscall wrappers (already has them)
2. ✅ Have command logic (can import from kernel or duplicate)
3. ✅ Use syscalls for all operations (already does)

**Note**: Userspace CLI currently has its own command implementations. 
In Phase 3, it can either:
- Keep its implementations (standalone)
- Call kernel syscalls that delegate to shell_commands (cleaner for future)

### Step 3: Update Kernel Boot Sequence
**Current**:
```rust
executor.spawn(Task::new(terminal()));
executor.spawn(Task::new(shell()));
```

**Target**:
```rust
executor.spawn(Task::new(terminal()));
// Load userspace CLI binary and execute as process
load_userspace_binary("orbital-cli", &mut executor);
```

### Step 4: Remove Kernel Shell Task
Once userspace shell is working:
- Delete `kernel/src/task/cli.rs`
- Delete `kernel/src/shell_commands.rs` (or keep as reference)
- Remove shell from `kernel/src/task/mod.rs`

## Implementation Plan

### Phase 3.1: Prepare Binary Embedding
1. Modify build system to embed userspace CLI binary
2. Add binary loader module to kernel
3. Test loading and executing embedded binary

### Phase 3.2: Execute Userspace Shell
1. Call binary loader on boot to launch userspace CLI
2. Userspace CLI reads from input buffer via sys_read(0)
3. Userspace CLI executes commands (local logic or syscalls)
4. Test all commands work from userspace

### Phase 3.3: Cleanup
1. Remove kernel shell task
2. Remove shell_commands.rs
3. Update documentation

## Syscall Requirements for Phase 3

Userspace shell will need these syscalls:
- ✅ `sys_read(0)` - Read keyboard input
- ✅ `sys_write(1)` - Write to stdout
- ✅ `sys_write(2)` - Write to stderr
- ✅ `sys_task_create` - Spawn processes
- ✅ `sys_task_wait` - Wait for process
- ✅ `sys_uptime` - Get kernel uptime
- ✅ `sys_get_pid` - Get current process ID
- ✅ `sys_ps` - List processes
- ✅ `sys_clear_screen` - Clear display
- ✅ `sys_run_ready` - Execute ready processes

All required syscalls are already implemented!

## Migration Path Example

**Step 1: Embed binary**
```rust
// In main.rs or boot sequence
let cli_binary = include_bytes!("../target/x86_64-unknown-linux-gnu/release/orbital-cli");
load_binary(cli_binary, "orbital-cli");
```

**Step 2: Load on boot**
```rust
// In Executor
executor.spawn(Task::new(terminal()));
// This will trigger binary loading and execution
```

**Step 3: Cleanup**
- Terminal still runs as kernel task (pure I/O)
- Shell runs as userspace process (command logic via syscalls)

## Benefits

✅ Shell runs in unprivileged mode (userspace)  
✅ Shell can crash without kernel crash  
✅ Clear mechanism/policy separation  
✅ Shell can be updated without kernel recompile  
✅ Foundation for multi-process system  
✅ Prepares for Phase 4: preemptive multitasking with multiple shells/services

## Timeline Estimate

- Step 1 (Binary loader): 2-3 hours
- Step 2 (Execute shell): 1-2 hours  
- Step 3 (Cleanup): 30 mins
- **Total**: ~4 hours

## Notes

- Keep terminal as kernel task for now (minimal I/O only)
- Input buffer mediates between kernel terminal and userspace shell
- All communication via syscalls and input buffer
- Shell can be easily replaced by running a different binary
- Prepares for multiple shells/services in future

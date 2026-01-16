# Orbital OS - Testing & Validation Guide

## Quick Start (Option A - Validation Testing)

This guide shows how to test the complete implementation:
- Interactive orbital-cli reading real keyboard input
- Kernel-userspace separation via syscalls
- Terminal task queuing input for userspace

## Prerequisites

- QEMU for x86_64 installed: `which qemu-system-x86_64`
- Rust nightly toolchain installed
- Cargo bootimage installed: `cargo install bootimage`

## Testing Steps

### 1. Build the Kernel

```bash
cd /Volumes/Works/Projects/orbital
cargo bootimage
```

This creates: `/target/x86_64-orbital/debug/bootimage-orbital.bin` (950KB)

### 2. Run in QEMU

```bash
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-orbital/debug/bootimage-orbital.bin \
  -m 256 \
  -cpu qemu64 \
  -nographic \
  -serial mon:stdio
```

Or use the VS Code task (Ctrl+Shift+B then select "Build & Run")

### 3. Interact with orbital-cli

Once booted, you'll see:

```
╔════════════════════════════════════════╗
║       Orbital CLI v0.1.0               ║
║  Userspace Policy via Kernel Syscalls  ║
╚════════════════════════════════════════╝
Type 'help' for available commands, 'exit' to quit.

> 
```

**Try these commands:**

```
help           # Show available commands
echo hello     # Echo text to stdout
echo foo bar   # Echo multiple arguments
exit           # Exit the CLI
```

### 4. What You're Testing

**Kernel Mechanisms:**
- ✅ Syscall dispatcher (sys_read, sys_write)
- ✅ Input buffer with lazy initialization
- ✅ Terminal task queuing keyboard input
- ✅ Process registry (ready for task_create)

**Userspace Policies:**
- ✅ Command parsing and execution
- ✅ Real stdin reading via sys_read
- ✅ stdout output via sys_write
- ✅ Interactive CLI loop

**Complete Pipeline:**
```
Keyboard Input
    ↓
Terminal Task (in kernel)
    ↓
Input Buffer (queues characters)
    ↓
orbital-cli (reads via sys_read)
    ↓
Command Dispatcher (userspace policy)
    ↓
Output via sys_write
```

## Testing Scenarios

### Scenario 1: Basic Command Execution
```
> help
Available Commands:
  help              - Show this help message
  echo <text>       - Echo text to stdout
  exit or quit      - Exit the CLI

> echo Hello from Orbital
Hello from Orbital

> exit
Goodbye!
```

### Scenario 2: Verify Separation of Concerns
- **Kernel doesn't know about CLI commands** - it just provides syscall mechanisms
- **Userspace provides the policy** - command parsing, help text, execution logic
- Type invalid commands to see userspace error handling:
  ```
  > invalid_command
  unknown command: 'invalid_command' (try 'help')
  ```

### Scenario 3: Check stdin/stdout Isolation
- Kernel provides sys_read(fd=0) for stdin, sys_write(fd=1) for stdout
- orbital-cli is a completely separate userspace program
- All I/O goes through syscalls - kernel never directly accesses CLI state

## Exit QEMU

Type `exit` in the CLI, or press `Ctrl+A X` (in QEMU monitor mode).

## Next Steps After Validation

### Option B: Task Execution (~20-30 hours)
- Implement sys_task_create for spawning new processes
- Add process context switching and scheduling
- Enable multi-tasking within the kernel

### Option C Phase 2-4: Enhanced CLI (~6+ hours)
- More syscall wrappers (sys_task_create, etc.)
- Better command parser with arguments
- Process listing and management commands

## Troubleshooting

**QEMU not found:**
```bash
# On macOS with Homebrew:
brew install qemu
```

**Bootimage build fails:**
```bash
# Update bootloader dependency
cargo update -p bootloader
cargo bootimage
```

**Kernel panics on boot:**
- Check that OnceCell lazy initialization is working
- Verify memory allocator isn't full
- All tests should pass: `cargo test`

## Architecture Reference

**Syscall Dispatch (kernel/src/syscall.rs):**
- Dispatcher at x86_64 syscall entry point
- 6 syscalls: hello(0), log(1), write(2), exit(3), read(4), task_create(5)
- Safe memory validation at boundary
- Error codes: -1 to -9

**Input System (kernel/src/input.rs):**
- 256-byte circular queue for stdin
- Lazy-initialized via OnceCell
- Populated by terminal task on keyboard input

**Terminal Task (kernel/src/task/terminal.rs):**
- Runs as async task in kernel
- Reads keyboard events from interrupt handler
- Echoes to VGA output
- Queues chars in input buffer for userspace

**CLI Program (userspace/cli/src/main.rs):**
- Pure userspace program using syscalls
- Implements policy: command parsing, help, echo
- Reads from stdin via sys_read
- Writes to stdout via sys_write

---

**Validation Complete When:**
- ✅ Bootimage builds cleanly
- ✅ QEMU boots without panic
- ✅ Terminal displays prompt
- ✅ Can type commands
- ✅ Commands execute correctly
- ✅ Output appears correctly
- ✅ Exit works as expected

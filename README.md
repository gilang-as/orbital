# Orbital OS

A lightweight, policy-free x86_64 operating system kernel built in Rust, demonstrating safe userspace-to-kernel communication through minimal syscalls.

**Status**: Active development - Phase 1: Core syscall infrastructure

## Quick Start

### Build
```bash
cargo bootimage
```

### Run in QEMU
```bash
cargo run
```

## Current Capabilities

### Kernel Features
- ✅ x86_64 bare-metal execution with VGA/serial output
- ✅ Memory management (paging, heap allocation with allocators)
- ✅ CPU initialization (GDT, IDT with interrupt handlers)
- ✅ Keyboard input with terminal echo
- ✅ Async task executor with keyboard/timer interrupts
- ✅ TTY abstraction layer (output routing)
- ✅ Syscall dispatcher with safe memory validation
- ✅ Process/task registry and lifecycle management

### Syscalls Implemented (6 total)
| # | Name | Status | Purpose |
|---|------|--------|---------|
| 0 | `sys_hello` | ✅ | Magic number validation test |
| 1 | `sys_log` | ✅ | Kernel logging with newline |
| 2 | `sys_write` | ✅ | UNIX-style write to fd (1=stdout, 2=stderr) |
| 3 | `sys_exit` | 🟡 | Process termination (stub) |
| 4 | `sys_read` | ✅ | Read from stdin (fd=0 only) |
| 5 | `sys_task_create` | ✅ | Spawn new process/task |

### Userspace Features
- ✅ Syscall wrappers with x86_64 inline assembly
- ✅ IPC module with syscall error handling
- ✅ CLI shell with commands: help, echo, ping, clear, spawn, ps
- ✅ Input buffer system for keyboard integration

## Architecture

Orbital OS follows a **hybrid kernel** design:
- **Kernel = Mechanism**: Provides syscalls, process management, memory/CPU primitives
- **Userspace = Policy**: Handles scheduling, priorities, application logic, system services

```
┌─────────────────────────────────────┐
│   Userspace Programs & Services     │
│   (orbital-cli, managementd, etc)   │
└──────────────┬──────────────────────┘
               │ Syscalls (ABI: x86_64 System V)
┌──────────────▼──────────────────────┐
│   Kernel (orbital-kernel)           │
│  - Syscall dispatcher               │
│  - Process/task registry            │
│  - Memory management                │
│  - I/O: VGA, Serial, Keyboard       │
│  - Async task executor              │
└─────────────────────────────────────┘
```

## Syscall ABI

**Calling Convention**: System V AMD64
- Arguments: RDI, RSI, RDX, RCX, R8, R9
- Syscall number: RAX (before call), also receives return value
- Instruction: `syscall` / `sysret`
- Error convention: Negative i64 (-1 to -9) indicates error

## Design Principles

1. **Policy-Free Kernel**: No system logic in kernel, only mechanisms
2. **Safe by Default**: Pointer and memory validation on syscall entry
3. **Minimal Overhead**: Lightweight process creation without execution overhead
4. **Separation of Concerns**: Clear boundaries between kernel and userspace

## Building for Development

### Prerequisites
- Rust nightly (`rustup update nightly`)
- QEMU (`brew install qemu` on macOS)
- Bootimage: `cargo install bootimage`

### Development Commands

```bash
# Build bootimage and run in QEMU
cargo bootimage
cargo run

# Build just the kernel
cargo build --lib -p orbital-kernel

# Check for errors without building
cargo check

# Run tests (integration tests via bootimage)
cargo test --test '*'

# View kernel documentation
cargo doc -p orbital-kernel --no-deps --open
```

## Project Structure

```
orbital/
├── kernel/               # Kernel library (no_std, bare-metal)
│   ├── src/
│   │   ├── main.rs      # Kernel entry point
│   │   ├── lib.rs       # Public kernel interface
│   │   ├── syscall.rs   # Syscall dispatcher & handlers
│   │   ├── process.rs   # Process registry & management
│   │   ├── input.rs     # Input buffer for stdin
│   │   ├── tty.rs       # Terminal abstraction layer
│   │   ├── gdt.rs       # Global Descriptor Table setup
│   │   ├── interrupts.rs# IDT & interrupt handlers
│   │   ├── vga_buffer.rs# VGA text output with cursor
│   │   ├── shell.rs     # Command dispatcher
│   │   ├── task/        # Async executor & tasks
│   │   └── ...
│   └── tests/           # Integration tests (bootimage)
│
├── boot/                 # Bootloader & early init (no_std)
│   └── src/main.rs      # Boot entry point
│
├── common/               # Shared types (no_std)
│   └── src/lib.rs       # Common structures & interfaces
│
├── userspace/
│   ├── ipc/             # IPC library with syscall wrappers (std)
│   ├── cli/             # Command-line interface (std)
│   └── managementd/     # Management daemon (planned)
│
├── docs/                # Architecture documentation
│   ├── Task_Launcher.md
│   ├── 13. Syscall Skeleton Design.md
│   ├── 12. IPC Transport Layer Design.md
│   └── ... (11 other design docs)
│
└── Cargo.toml           # Workspace configuration
```

## Kernel Shell Commands

Interactive shell running in kernel with these commands:

```
help              - Show available commands
echo <msg>        - Print a message
ping              - Respond with pong
spawn             - Create a new process (demo)
ps                - List all processes
clear             - Clear the screen
```

## Testing

Integration tests verify:
- ✅ Basic kernel boot
- ✅ Heap allocation
- ✅ Stack overflow interrupt handling
- ✅ Panic propagation

```bash
# Run all tests
cargo test --test '*'

# Run specific test
cargo test --test basic_boot
```

## Current Limitations

1. **No process execution**: Processes are created but don't run
2. **Single address space**: No memory isolation
3. **Blocking I/O only**: No async syscalls yet
4. **Limited error codes**: 9 error types (-1 to -9)
5. **No signals**: No event delivery to processes
6. **No IPC**: Inter-process communication not yet implemented

## Next Steps

### Phase 2: Task Execution
- Wire process entry points to async executor
- Implement userspace task execution
- Add task scheduling (round-robin or priority-based)

### Phase 3: Memory Isolation
- Implement paging for memory protection
- Add task-local virtual address spaces
- Implement fork/exec syscalls

### Phase 4: IPC & Services
- Implement ring buffer-based IPC
- Add management daemon
- Implement service discovery

### Phase 5: Advanced Features
- Networking (raw sockets, TCP/IP)
- Package system & package manager
- RBAC & capability-based security
- File system (minimal)

## Documentation

See `docs/` directory for detailed architecture:
- [Task_Launcher.md](docs/Task_Launcher.md) - Process management design
- [Syscall Skeleton Design.md](docs/13.%20Syscall%20Skeleton%20Design.md) - Syscall ABI
- [IPC Transport Layer Design.md](docs/12.%20IPC%20Transport%20Layer%20Design.md) - IPC architecture
- REFACTORING.md - Recent refactoring notes
- WORKSPACE.md - Crate organization

## Development Notes

- All code is Rust, using nightly features
- Kernel is `#![no_std]` with custom allocators
- Userspace is standard library (std)
- Integration tests run via `bootimage` test runner
- Target: x86_64-unknown-none-gnu (bare-metal, no hosted OS)

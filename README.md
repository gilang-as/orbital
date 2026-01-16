# Orbital OS

A production-grade, modular network operating system built from scratch with a focus on security, observability, and modular architecture.

**Status**: Early development - Architecture refactoring phase

## Quick Start

### Build
```bash
cargo build
```

### Run in QEMU
```bash
cargo run
```

### Run Tests
```bash
cargo test
```

## Architecture

Orbital OS is built as a Cargo workspace with clear separation of concerns:

- **`kernel/`**: Core OS functionality (bootloader + kernel code)
- **`boot/`**: Entry point and early initialization
- **`common/`**: Shared types and interfaces
- **`userspace/managementd/`**: System management daemon (planned)
- **`userspace/ipc/`**: Inter-process communication (planned)

See [WORKSPACE.md](WORKSPACE.md) for detailed architecture documentation.

## Design Principles

1. **No Kernel Logic**: All system features belong in userspace
2. **IPC-First Design**: All system control through management daemon
3. **Modular Crates**: Clear boundaries between components
4. **Type-Safe Interfaces**: Shared types only in `common/` crate
5. **Security by Design**: RBAC and capabilities at the foundation

## Current Features

- [x] Basic bootloader integration
- [x] VGA text output
- [x] Memory management (paging, heap allocation)
- [x] CPU initialization (GDT, IDT)
- [x] Task/async execution
- [x] Keyboard input with echo command
- [ ] IPC infrastructure
- [ ] Management daemon
- [ ] Networking
- [ ] Package system
- [ ] RBAC & capabilities

## Building for Development

### Prerequisites
- Rust nightly
- QEMU
- Bootimage tool: `cargo install bootimage`

### Development Commands
```bash
# Build kernel only
cargo build -p orbital-kernel

# Build boot crate
cargo build -p orbital-boot

# Run all tests
cargo test

# Run kernel tests only
cargo test -p orbital-kernel

# Build documentation
cargo doc --open
```

## Testing

The kernel includes integration tests:
- `basic_boot`: Verify basic kernel initialization
- `should_panic`: Test panic handling
- `stack_overflow`: Test interrupt handling with stack overflow
- `heap_allocation`: Test heap allocator

Run tests with:
```bash
cargo test --test '*' -- --nocapture
```

## Workspace Structure

```
orbital/
├── kernel/          # Kernel library (no_std)
├── boot/            # Bootloader & init (no_std)
├── common/          # Shared types (no_std)
├── userspace/       # User-facing services (std)
├── docs/            # Architecture documentation
└── tests/           # Integration tests
```

# Orbital OS - Workspace Architecture

This document describes the Cargo workspace structure for Orbital OS, a production-grade, modular network operating system.

## Workspace Structure

```
orbital/
├── Cargo.toml              # Workspace root
├── kernel/                 # Kernel crate (no_std)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs         # Kernel library entry point
│   │   ├── allocator.rs
│   │   ├── gdt.rs
│   │   ├── interrupts.rs
│   │   ├── memory.rs
│   │   ├── serial.rs
│   │   ├── shell.rs
│   │   ├── vga_buffer.rs
│   │   ├── task/          # Task/executor modules
│   │   └── allocator/     # Memory allocator implementations
│   └── tests/             # Integration tests
├── boot/                   # Boot/firmware crate (no_std)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs        # Entry point, initializes kernel
├── common/                 # Shared types (no_std)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs         # IPC types, error definitions
├── userspace/
│   ├── managementd/       # Management daemon (std)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── ipc/               # IPC library (no_std)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
└── docs/                   # Architecture documentation
```

## Crate Responsibilities

### `kernel` - Orbital Kernel (`orbital-kernel`)
- **Type**: Library (no_std)
- **Role**: Core kernel functionality
- **Exports**: Hardware initialization, memory management, task scheduling, drivers
- **Key Modules**:
  - `gdt`: Global Descriptor Table setup
  - `interrupts`: CPU interrupt handlers
  - `memory`: Virtual memory and paging
  - `allocator`: Heap allocation strategies
  - `task`: Async task execution and scheduling
  - `vga_buffer`: Text display output
  - `serial`: Serial port I/O
  - `shell`: Command parsing (temporary - will move to userspace)

**Important**: The kernel exports only low-level abstractions and does NOT contain:
- Business logic
- User-facing commands (except shell as a temporary placeholder)
- Complex state management
- System policies

All system control and policy logic belongs in userspace via the management daemon.

### `boot` - Bootloader & Initialization (`orbital-boot`)
- **Type**: Binary (no_std)
- **Role**: Firmware entry point and early initialization
- **Entry**: `boot_main()` called by bootloader
- **Responsibilities**:
  1. Parse bootloader memory map
  2. Initialize kernel subsystems
  3. Set up virtual memory
  4. Initialize heap allocator
  5. Spawn user task executor
  6. Spawn management daemon connection (stub)

**Future**: Once we have std support, boot will spawn:
- The management daemon in user mode
- Other system services

### `common` - Shared Types (`orbital-common`)
- **Type**: Library (no_std)
- **Role**: Type definitions used across crates
- **Contains**: 
  - IPC message types (`MgmtCommand`, `MgmtResponse`)
  - Error types (`OrbitalError`)
  - Shared configuration structures

**Rule**: No implementation logic. Only type definitions and traits.

### `userspace/ipc` - IPC Library (`orbital-ipc`)
- **Type**: Library (no_std, but designed for std)
- **Role**: Inter-process communication abstractions
- **Exports**:
  - `IpcClient`: Send commands to daemon
  - `IpcServer`: Receive commands in daemon
- **Implementation**: Currently stubs; will use Unix Domain Sockets

### `userspace/managementd` - Management Daemon (`orbital-managementd`)
- **Type**: Binary (std)
- **Role**: System state management and policy enforcement
- **Responsibilities**:
  - Accept configuration requests via IPC
  - Manage system state
  - Enforce security policies (RBAC, capabilities)
  - Interface with kernel via syscalls
  - Control lifecycle of services
- **Current Status**: Stub; prints message on startup

## Build Configuration

- **Workspace Target**: `x86_64-orbital.json` (bare-metal x86_64)
- **Bootloader**: bootloader v0.9 with physical memory mapping
- **Default Member**: `boot` (runs via `cargo run`)
- **Test Framework**: Bootimage integration tests + QEMU
- **Testing**: Unit tests disabled for bare-metal crates

### Workspace Configuration
The workspace Cargo.toml specifies:
- **members**: kernel, boot, common (userspace in separate workspace later)
- **resolver**: version 2 (required for workspace inheritance)
- **default-members**: boot (primary binary target)

### Test Configuration
- `orbital-kernel`: Integration tests run via bootimage + QEMU
- `orbital-boot`: No tests (firmware entry point)
- `orbital-common`: Tests disabled (bare-metal no_std library)

## Development Workflow

### Building
```bash
cargo build              # Build default (boot) binary
cargo build -p orbital-kernel  # Build kernel library
cargo build --workspace  # Build all crates in workspace
```

### Running
```bash
cargo run              # Run boot binary in QEMU (default)
cargo run --bin orbital  # Explicit binary name
```

### Testing
```bash
cargo test             # Run bootimage integration tests
cargo test -p orbital-kernel  # Test kernel package
cargo test -- --nocapture  # Show test output
```

### Documentation
```bash
cargo doc -p orbital-kernel --no-deps  # Generate kernel docs
cargo doc -p orbital-kernel --no-deps --open  # Open in browser
```

Note: Full `cargo doc` not supported for bare-metal target (dependencies can't be documented).

## Design Principles

1. **Separation of Concerns**: Kernel = hardware, Boot = initialization, Userspace = policy
2. **No Kernel Logic**: All features belong in userspace services
3. **IPC Boundaries**: Enforced through type system and separate crates
4. **Stubbed Interfaces**: All future features have skeleton implementations
5. **No Direct System State**: Only through management daemon
6. **Modular Testing**: Each crate testable according to its constraints

## Next Steps

1. **Implement IPC**: Replace stubs with Unix Domain Socket implementation
2. **Add std Support**: Build userspace crates for host target
3. **Implement Management Daemon**: Accept and process commands
4. **Extract Shell**: Move terminal/shell to userspace CLI tool
5. **Add Networking**: Implement network data plane in separate crate
6. **Add Security**: Implement RBAC and capability system

## Troubleshooting

### `cargo test` fails with "can't find crate for `test`"
- This is expected for `no_std` crates
- Solution: Run `cargo test -p orbital-kernel` to test specific packages
- Or use `cargo run` with bootimage for integration tests

### `cargo doc --open` panics
- Bare-metal target can't document dependencies
- Solution: Use `cargo doc -p orbital-kernel --no-deps --open`

### `cargo run` says "could not determine which binary to run"
- Solution: Already fixed! Default member is `boot`
- Or explicitly: `cargo run --bin orbital`

## References

- `docs/1. Kernel Foundation Documentation.md` - Low-level kernel design
- `docs/8. Ipc & Api Design.md` - IPC protocol specification
- `docs/9. Security Model (RBAC & Capabilities).md` - Security architecture

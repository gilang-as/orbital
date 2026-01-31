# Orbital OS

A bare-metal x86_64 operating system kernel written in Rust with an interactive userspace shell.

---

## Status

| Attribute | Value |
|-----------|-------|
| Phase | 11 - Functional Interactive Shell |
| Syscalls | 12 implemented |
| Shell Commands | 7 functional |
| Processes | 3 concurrent shells |
| Scheduling | Cooperative (async/await) |

---

## Quick Start

**Requirements**: Rust nightly, QEMU, bootimage

```bash
# Install bootimage
cargo install bootimage

# Build and run
cargo run

# Run tests
cargo test
```

---

## What's Implemented

- x86_64 bare-metal boot with VGA and serial output
- Memory management: paging, heap allocation
- Interrupt handling: timer (~100 Hz), keyboard
- Process management with async executor
- 12 syscalls for kernel-userspace communication
- Interactive shell with 7 commands (help, echo, pid, uptime, ps, clear, exit)

## Not Implemented

- Memory isolation between processes
- Preemptive scheduling
- File system
- Networking
- IPC message passing
- Signal handling

---

## Documentation

| Topic | Document |
|-------|----------|
| Documentation Index | [docs/README.md](docs/README.md) |
| System Architecture | [docs/architecture/overview.md](docs/architecture/overview.md) |
| Syscall Reference | [docs/reference/syscall-table.md](docs/reference/syscall-table.md) |
| Shell Commands | [docs/reference/shell-commands.md](docs/reference/shell-commands.md) |
| Code Map | [docs/reference/code-map.md](docs/reference/code-map.md) |
| Development Phases | [docs/development/phases.md](docs/development/phases.md) |
| Future Vision | [docs/vision/README.md](docs/vision/README.md) |

---

## Project Structure

```
orbital/
├── boot/          # Bootloader entry point (no_std binary)
├── kernel/        # Kernel library (no_std)
├── common/        # Shared types (no_std)
├── userspace/     # Userspace programs
│   └── minimal/   # Interactive shell
└── docs/          # Documentation
```

---

## For AI Agents

This repository follows AI-first documentation conventions. See [CLAUDE.md](claude.md) for interaction rules.

**Entry points**:
- [docs/README.md](docs/README.md) - Navigate all documentation
- [docs/reference/code-map.md](docs/reference/code-map.md) - Find code by function

**Key constraints**:
- Kernel and userspace are `no_std` (no standard library)
- Single address space (no memory isolation)
- Cooperative scheduling only
- Shell has no heap allocation

---

## License

[Specify license here]

---

**Last Updated**: January 2026

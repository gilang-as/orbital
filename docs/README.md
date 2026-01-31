# Orbital OS Documentation

**Purpose**: Entry point for all Orbital OS documentation
**Scope**: Navigation and document classification
**Last Updated**: January 2026
**Implementation Status**: Phase 11 - Functional Interactive Shell

---

## Quick Navigation

| Need | Document |
|------|----------|
| Project overview | [README.md](../README.md) |
| System architecture | [architecture/overview.md](architecture/overview.md) |
| Syscall reference | [reference/syscall-table.md](reference/syscall-table.md) |
| Shell commands | [reference/shell-commands.md](reference/shell-commands.md) |
| Code navigation | [reference/code-map.md](reference/code-map.md) |
| Development history | [development/phases.md](development/phases.md) |
| Future plans | [vision/README.md](vision/README.md) |

---

## Documentation Structure

```
docs/
├── README.md                    # This file (navigation)
├── architecture/                # IMPLEMENTED FEATURES
│   └── overview.md              # Current system architecture
├── development/                 # DEVELOPMENT INFO
│   └── phases.md                # Phase history and roadmap
├── reference/                   # QUICK REFERENCE
│   ├── syscall-table.md         # All 12 syscalls
│   ├── shell-commands.md        # All 7 commands
│   └── code-map.md              # File-to-function map
├── vision/                      # FUTURE/ASPIRATIONAL (not implemented)
│   ├── README.md                # Vision disclaimer
│   └── *.md                     # Future architecture plans
├── kernel/                      # KERNEL SPECIFICS
│   └── tty.md                   # TTY subsystem
├── syscalls/                    # SYSCALL DETAILS
│   ├── sys_log.md               # sys_log documentation
│   └── sys_write.md             # sys_write documentation
└── userspace/                   # USERSPACE SPECIFICS
    └── cli.md                   # CLI framework
```

---

## Document Classification

### AUTHORITATIVE (reflects current code)

| Document | Covers |
|----------|--------|
| architecture/overview.md | System layers, data flow, process model |
| reference/syscall-table.md | All 12 syscalls with args/returns |
| reference/shell-commands.md | All 7 shell commands |
| reference/code-map.md | Every source file mapped |
| development/phases.md | Completed phases 1-11, planned phases |

### REFERENCE (specific subsystems)

| Document | Covers |
|----------|--------|
| kernel/tty.md | TTY output abstraction |
| syscalls/sys_log.md | sys_log syscall details |
| syscalls/sys_write.md | sys_write syscall details |
| userspace/cli.md | CLI framework (needs update) |

### VISION (NOT IMPLEMENTED)

All documents in `vision/` describe **future architecture**.
See [vision/README.md](vision/README.md) for disclaimer.

---

## Current Implementation Status

### IMPLEMENTED (Phase 11)

- 12 syscalls operational
- 3 concurrent shell processes
- 7 shell commands functional
- ELF binary loading
- Cooperative multitasking
- VGA + serial output
- Keyboard input

### NOT IMPLEMENTED

- Memory isolation between processes
- Preemptive scheduling
- File system
- Networking
- IPC transport
- Signal handling
- RBAC/capabilities

---

## For AI Agents

### Finding Information

1. **What does file X do?** → [reference/code-map.md](reference/code-map.md)
2. **How does syscall Y work?** → [reference/syscall-table.md](reference/syscall-table.md)
3. **What's implemented?** → [architecture/overview.md](architecture/overview.md)
4. **What's the roadmap?** → [development/phases.md](development/phases.md)
5. **What's NOT implemented?** → [vision/README.md](vision/README.md)

### Key Constraints

- Kernel is `no_std` (no standard library)
- Userspace shell is `no_std` (no heap)
- Single address space (no memory isolation)
- Cooperative scheduling only (no preemption)
- 3 hardcoded shell processes

### Code Entry Points

| Purpose | File | Function |
|---------|------|----------|
| Boot | boot/src/main.rs | `boot_main()` |
| Syscall dispatch | kernel/src/syscall.rs | `dispatch_syscall()` |
| Shell entry | userspace/minimal/src/main.rs | `_start()` |
| Process creation | kernel/src/process.rs | `create_process()` |

---

## Validation

All documentation validated against codebase January 2026.

To reverify:
1. Check syscall count: `grep -c "^fn sys_" kernel/src/syscall.rs`
2. Check process count: `grep "spawn_userspace_task" kernel/src/multiprocess.rs`
3. Check shell commands: `grep "else if" userspace/minimal/src/main.rs | wc -l`

---

**Document Status**: AUTHORITATIVE
**Supersedes**: All previous documentation indexes

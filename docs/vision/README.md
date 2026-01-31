# Orbital OS - Vision Documents

**IMPORTANT DISCLAIMER**

The documents in this directory describe **aspirational architecture** that is **NOT YET IMPLEMENTED**.

---

## Document Status

| Document | Implementation Status |
|----------|----------------------|
| kernel-vision.md | PARTIAL - Core kernel works, not all features |
| userspace-vision.md | STUB - Only minimal shell implemented |
| configuration.md | NOT IMPLEMENTED |
| packages.md | NOT IMPLEMENTED |
| networking.md | NOT IMPLEMENTED |
| management.md | STUB - Daemon exists but non-functional |
| updates.md | NOT IMPLEMENTED |
| ipc.md | STUB - Types defined, no transport |
| security.md | NOT IMPLEMENTED |
| observability.md | NOT IMPLEMENTED |
| ipc-transport.md | NOT IMPLEMENTED |
| syscall-boundary.md | PARTIAL - Syscalls work, not all features |
| syscall-skeleton.md | OUTDATED - Shows 6 syscalls, now 12 |

---

## How to Use These Documents

1. **For Planning**: Use as reference for future implementation
2. **For Context**: Understand the long-term vision
3. **NOT for Current State**: See `docs/architecture/` for what IS implemented

---

## What IS Implemented (Phase 11)

See [docs/architecture/overview.md](../architecture/overview.md) for current state:

- Basic kernel with syscalls
- Cooperative multitasking
- 3 concurrent shell processes
- 12 syscalls
- No memory isolation
- No networking
- No file system
- No IPC transport

---

## Vision vs Reality Gap

| Vision Document Claim | Current Reality |
|----------------------|-----------------|
| "Preemptive scheduler" | Cooperative only |
| "Memory isolation" | Single address space |
| "IPC message passing" | Stub types only |
| "RBAC & capabilities" | Not implemented |
| "Package system" | Not implemented |
| "Networking fast-path" | Not implemented |
| "Configuration system" | Not implemented |

---

## When Vision Becomes Reality

These documents will be updated and moved to `docs/architecture/` as features are implemented:

1. Implement feature
2. Verify against vision document
3. Update document to reflect actual implementation
4. Move to `docs/architecture/`
5. Mark vision document as superseded

---

**Last Updated**: January 2026
**Current Phase**: 11

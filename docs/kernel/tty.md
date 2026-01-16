# TTY (Teletype) Device Primitive

## Overview

The TTY module provides a minimal **terminal device abstraction** that decouples logical output (stdout, stderr) from physical backends (serial port, VGA buffer). It is a kernel **primitive only** — no policy, buffering, line discipline, or control sequences.

## Purpose

### Current Problem

Before TTY abstraction, `sys_write()` directly manipulated the serial port:

```
userspace syscall → sys_write() → serial port write → output
```

This created tight coupling:
- Output logic duplicated in `sys_log()` and `sys_write()`
- Difficult to swap backends (serial ↔ VGA)
- No device abstraction concept in kernel
- Unclear ownership of serial lock

### TTY Solution

Now `sys_write()` and `sys_log()` route through a device abstraction:

```
userspace syscall → sys_write() → tty_write() → serial port write → output
kernel log        → sys_log()    → tty_write_with_newline() → output
```

Benefits:
- **Decoupled output logic** — one place to change backends
- **Device abstraction** — establishes kernel primitive for "device"
- **Extensible** — future: VGA, network, storage backends
- **Tested independently** — TTY can be tested separately from syscalls

## Architecture

### Module: `kernel/src/tty.rs`

**Public Functions:**

| Function | Purpose | Semantics |
|----------|---------|-----------|
| `tty_write(buf: &[u8]) -> usize` | Write raw bytes to TTY | No modification, no newline |
| `tty_write_with_newline(buf: &[u8]) -> usize` | Write bytes + newline | For kernel logging |

**Internal Implementation:**

- Disables interrupts during write (atomic operation)
- Locks serial port mutex
- Routes all writes to serial (UART 0x3F8)
- No buffering, line discipline, or flow control

### Integration Points

**In `sys_write()` (fd=1, fd=2):**
```rust
crate::tty::tty_write(&buffer);  // No newline - userspace controls format
```

**In `sys_log()` (internal kernel logging):**
```rust
crate::tty::tty_write_with_newline(&buffer);  // Adds newline for readability
```

## Design Constraints

### What TTY Does

✓ Routes bytes to physical devices
✓ Provides atomic write (interrupts disabled)
✓ Validates buffer size (max 4096 bytes)
✓ Serializes access (serial port mutex)
✓ Preserves exact byte content (no filtering)

### What TTY Does NOT Do

✗ Buffer data (no internal queue)
✗ Implement line discipline (no \r\n conversion)
✗ Parse escape sequences (no ANSI support)
✗ Manage terminal state (no width/height/modes)
✗ Handle input (no keyboard → TTY)
✗ Add timestamps or prefixes to output
✗ Implement paging or scrollback

## Safety Properties

### Interrupt Safety

TTY disables interrupts during the critical section (serial write). This prevents:
- Context switches mid-write
- Interleaved output from different sources
- Corruption of serial port state

### Memory Safety

- Caller is responsible for validating buffer contents
- TTY does not interpret or modify bytes
- Maximum size enforced (4096 bytes)
- No overflow possible

### Concurrency

Serial port protected by spin lock (`crate::serial::SERIAL1`). TTY serializes all writes atomically.

## Current Limitations

### Single Backend

All output routes to serial port. VGA buffer code exists but disabled to avoid:
- Display corruption during kernel output
- Conflict with VGA scrolling/clearing
- Unnecessary complexity

### No Input

TTY is write-only. Input handling (keyboard, etc.) managed separately by task system.

### No Buffering

Each `tty_write()` call directly writes to hardware. For high-frequency logging, consider:
- Ring buffer in userspace
- Batch writes before syscall
- Dedicated log collector daemon

### Fixed Size Limit

Maximum 4096 bytes per write. Matches `sys_write()` validation.

## Future Directions

### Phase 2: Extensible Backends

Add trait-based backend selection:

```rust
pub trait TtyBackend: Send {
    fn write(&mut self, buf: &[u8]) -> usize;
}

pub struct SerialBackend { /* ... */ }
pub struct VgaBackend { /* ... */ }
pub struct RingBufferBackend { /* ... */ }

static TTY_BACKEND: Mutex<Box<dyn TtyBackend>> = ...;
```

### Phase 3: VGA Support

Enable VGA as primary output to avoid serial port latency.

### Phase 4: Kernel Console

Multiple outputs simultaneously:
- Primary: Serial (kernel debugging)
- Secondary: VGA (user visibility)
- Tertiary: Network (remote logging)

### Phase 5: Input Integration

- `tty_read()` for keyboard input
- TTY as terminal device (not just output)
- Line buffering for interactive shells

## Testing

TTY module includes unit tests:

- `test_tty_write_empty()` — zero-length write
- `test_tty_write_single_byte()` — single byte
- `test_tty_write_multiple_bytes()` — normal case
- `test_tty_write_max_size()` — boundary condition
- `test_tty_write_exceeds_max()` — validation (panic)
- `test_tty_write_with_newline()` — logging variant

Run via: `cargo test --lib kernel::tty`

## Usage Examples

### Userspace: Writing to stdout

```rust
// userspace program
let message = b"Hello, World!\n";
let fd = 1; // stdout
syscall_write(fd, message.as_ptr(), message.len())
  → kernel sys_write(fd=1, ptr, len)
    → crate::tty::tty_write(&buffer)
      → serial port: "Hello, World!\n" (no extra newline)
```

### Userspace: Writing to stderr

```rust
let error = b"Error occurred";
let fd = 2; // stderr
syscall_write(fd, error.as_ptr(), error.len())
  → kernel sys_write(fd=2, ptr, len)
    → crate::tty::tty_write(&buffer)
      → serial port: "Error occurred" (no extra newline)
```

### Kernel: Logging a message

```rust
// kernel code (sys_log)
let message = b"Interrupt 0x21 received";
sys_log(message.as_ptr(), message.len(), ...)
  → crate::tty::tty_write_with_newline(&buffer)
    → serial port: "Interrupt 0x21 received\n" (added by TTY)
```

## Summary Table

| Aspect | TTY | sys_write | sys_log |
|--------|-----|-----------|---------|
| **Purpose** | Device abstraction | UNIX write syscall | Kernel logging |
| **Input** | Raw bytes | Userspace buffer (fd specified) | Kernel buffer |
| **Output** | Serial (or future backend) | Serial via TTY | Serial via TTY |
| **Newline Handling** | No automatic newline | User controls | TTY adds newline |
| **Caller** | sys_write, sys_log, kernel code | Userspace | Kernel |
| **Atomicity** | Interrupts disabled | Implicit via tty_write | Implicit via tty_write_with_newline |

## References

- [Syscall Design](../13.%20Syscall%20Skeleton%20Design.md) — Syscall architecture overview
- [Syscall Spec: sys_write](../syscalls/sys_write.md) — UNIX write(2) specification
- [Syscall Spec: sys_log](../syscalls/sys_log.md) — Internal logging specification

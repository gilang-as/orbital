# Syscall Skeleton Design

**Status:** Minimal Implementation  
**Date:** January 16, 2026  
**Version:** 1.0  
**Scope:** Syscall entry mechanism, dispatcher, and 3 test syscalls

---

## 1. Overview

The syscall skeleton provides the foundational mechanism for userspace processes to request kernel services. It is **not yet fully wired up** but provides:

- Syscall dispatcher and routing
- Error handling convention
- Minimal test syscalls for validation
- Architecture for future syscall implementation

This design focuses on the **mechanism**, not the policy. Policy (who can call which syscalls, what quotas apply, etc.) belongs in userspace.

---

## 2. Architecture

### 2.1 Entry Point (Not Yet Implemented)

The x86_64 `syscall` instruction (0x0F 0x05) is used to enter the kernel. This instruction:
- Does **not** use the IDT (Interrupt Descriptor Table)
- **Does** use Model-Specific Registers (MSRs):
  - `IA32_LSTAR` (0xC0000082): Kernel entry point address
  - `IA32_STAR` (0xC0000081): Segment selectors
  - `IA32_FMASK` (0xC0000084): RFLAGS mask for syscall

When `syscall` executes from userspace:
```
Before syscall (userspace):
  RAX = syscall number (0, 1, 2, ...)
  RDI = argument 1
  RSI = argument 2
  RDX = argument 3
  RCX = argument 4  (Note: clobbered on entry)
  R8  = argument 5
  R9  = argument 6

Syscall instruction transitions:
  RCX = return address (saved from RIP)
  R11 = RFLAGS (saved for restoration)
  CS:RIP = kernel entry point (from IA32_LSTAR)
  RFLAGS = masked with IA32_FMASK

After sysret (back to userspace):
  RIP = value from RCX
  RFLAGS = value from R11
```

### 2.2 Entry Assembly (Stub Location)

The entry point assembly should:

1. **Save userspace context:**
   - RCX (return address), R11 (RFLAGS) are automatically saved by `syscall`
   - Save RSP (user stack pointer) if kernel uses separate stacks

2. **Set up kernel environment:**
   - Load kernel stack pointer
   - Load kernel GDT/LDT if needed
   - Save user registers for later restoration

3. **Call C-level dispatcher:**
   - `dispatch_syscall(rax, rdi, rsi, rdx, rcx, r8, r9)` → returns i64

4. **Restore and return:**
   - Place return value in RAX
   - Restore user context
   - Execute `sysret` (automatically restores RCX→RIP, R11→RFLAGS)

**Status:** TODO in phase 2. Currently the dispatcher exists but no assembly entry point.

### 2.3 Dispatcher

The kernel module `kernel::syscall::dispatch_syscall` routes syscalls to handlers:

```
dispatch_syscall(
    syscall_nr: usize,      // RAX from userspace
    arg1: usize,             // RDI
    arg2: usize,             // RSI
    arg3: usize,             // RDX
    arg4: usize,             // RCX
    arg5: usize,             // R8
    arg6: usize,             // R9
) → i64                      // Return value for RAX
```

The dispatcher:
- Indexes into `SYSCALL_TABLE`
- Calls handler if present, returns error code otherwise
- Converts `Result<usize, SysError>` to `i64` (negative = error)

### 2.4 Syscall Table

A static array mapping syscall numbers to handler functions:

```
SYSCALL_TABLE[0] = Some(sys_hello)
SYSCALL_TABLE[1] = Some(sys_log)
SYSCALL_TABLE[2] = Some(sys_exit)
SYSCALL_TABLE[3..] = None
```

Entries are `Option<fn(usize, usize, usize, usize, usize, usize) -> SysResult>`.

---

## 3. Error Handling

### 3.1 Error Codes

Syscall errors are represented as negative i64 values (Unix convention):

| Error | Code | Meaning |
|-------|------|---------|
| `Invalid` | -1 | Invalid argument or syscall number |
| `NotImplemented` | -2 | Syscall not yet implemented |
| `Fault` | -3 | Memory fault (invalid pointer) |
| `PermissionDenied` | -4 | Permission denied |
| `NotFound` | -5 | Resource not found |
| `Error` | -6 | Generic kernel error |

### 3.2 Return Value Convention

**Success:**
```
RAX = result value (≥ 0)
```

**Failure:**
```
RAX = negative error code (-1 to -6)
```

Userspace interprets negative RAX as error and looks up the error code.

### 3.3 No Exceptions on Error

Unlike traditional Unix `errno`, syscall errors are returned directly in RAX. There are:
- No exceptions thrown
- No signal handlers invoked
- No process termination
- No kernel panics

The kernel always returns control to userspace with a result.

---

## 4. Minimal Syscalls

### 4.1 sys_hello(0) - Test Syscall

**Purpose:** Verify syscall mechanism works

**Arguments:**
- `arg1` (RDI): Magic number (0xCAFEBABE for success)

**Return value:**
- Success: 0xDEADBEEF
- Failure: -1 (Invalid argument)

**Implementation:**
- Check if magic number matches
- No side effects
- No state changes

**Use case:** Bootloader or test suite validates syscall entry point is wired correctly.

### 4.2 sys_log(1) - Write to Kernel Log

**Purpose:** Allow userspace to write diagnostic messages to kernel log

**Arguments:**
- `arg1` (RDI): Pointer to message buffer (user memory)
- `arg2` (RSI): Message length in bytes (≤ 1024)

**Return value:**
- Success: number of bytes written
- Failure: -1 (Invalid), -3 (Fault), -6 (Error)

**Validation:**
- Message length must be 1-1024 bytes
- Pointer must be in user memory space (not validated in stub)
- In full implementation, bounds check: `ptr + len ≤ user_memory_end`

**Implementation stub:**
- Accepts pointer and length
- Does NOT actually write (would require user memory validation)
- Returns length on success

**Future:** Full implementation reads from user memory and writes to kernel VGA buffer or serial port.

### 4.3 sys_exit(2) - Terminate Process

**Purpose:** Allow process to exit cleanly

**Arguments:**
- `arg1` (RDI): Exit code (32-bit signed int)

**Return value:**
- Never returns (process terminates)
- On error (already exiting): -2 (NotImplemented)

**Implementation:**
- Mark current task as exiting
- Free task resources (memory, file descriptors, etc.)
- Reschedule to next task
- Never returns to userspace

**Status:** Stub returns NotImplemented (full task scheduler needed).

---

## 5. Argument Passing

Syscalls follow System V AMD64 ABI for argument passing:

| Argument | Register | Preserved? |
|----------|----------|-----------|
| arg1 | RDI | No (clobbered) |
| arg2 | RSI | No (clobbered) |
| arg3 | RDX | No (clobbered) |
| arg4 | RCX | No (clobbered on entry) |
| arg5 | R8 | No (clobbered) |
| arg6 | R9 | No (clobbered) |
| Return | RAX | - |

**Note:** RCX is clobbered by the `syscall` instruction itself (saved to by kernel). Callers must be aware the 4th argument (in RCX) is lost after the syscall completes.

---

## 6. Userspace Wrappers

The `userspace/ipc` crate provides C-level wrapper functions that will eventually invoke the syscall instruction via inline assembly:

```rust
pub fn syscall_hello(magic: u64) -> SyscallResult<u64>
pub fn syscall_log(ptr: *const u8, len: usize) -> SyscallResult<usize>
pub fn syscall_exit(exit_code: i32) -> SyscallResult<!>
```

**Current status:** Stubs (return NotImplemented or panic). Full implementation awaits assembly entry point.

**Future:** Will use inline assembly:
```
asm!("syscall",
    inout("rax") syscall_nr => result,
    in("rdi") arg1,
    in("rsi") arg2,
    ... (other args)
    clobber_abi("C"),  // Tell compiler C calling convention is clobbered
)
```

---

## 7. Not Yet Implemented

### 7.1 Assembly Entry Point

The actual `syscall` instruction handler is not yet implemented. Needed:
- Assembly entry point at address stored in `IA32_LSTAR`
- Context save/restore code
- Stack switching (user stack → kernel stack)
- Inline assembly in userspace wrappers

### 7.2 Memory Validation

The `sys_log` syscall does not validate user pointers. Full implementation must:
- Check pointer is in user memory range
- Prevent out-of-bounds reads
- Handle page faults gracefully

### 7.3 Task Context

The syscalls don't yet have access to task context (task ID, memory bounds, etc.). Full implementation requires:
- Task structure with memory layout information
- CPU register saving per task
- Userspace memory bounds tracking

### 7.4 Preemption and Context Switching

The syscalls are not preemptive. Future work:
- Implement task scheduler
- Handle syscall as interruptible operation
- Support context switching during syscalls

---

## 8. Design Principles

### Principle 1: Minimal Kernel

The syscall dispatcher and handlers are deliberately small:
- 3 minimal syscalls only
- No complex logic
- No policy enforcement
- No performance optimization

Each syscall is ≤ 20 lines of code.

### Principle 2: Policy in Userspace

The kernel does not decide:
- Who can call syscalls (permission checking)
- How many syscalls per second (rate limiting)
- Which syscalls are available (configuration)
- How to handle errors (error policy)

All policy moves to userspace.

### Principle 3: Non-Blocking

Syscalls return immediately with a result. There is:
- No waiting in kernel
- No blocking on resources
- No timeout logic
- No condition variables in kernel

Blocking policies move to userspace.

### Principle 4: Explicit Error Codes

Syscalls return errors directly in RAX. There is:
- No exception mechanism
- No signal delivery
- No side effects on error
- No hidden state (like errno)

Userspace sees the exact error code and decides what to do.

---

## 9. Future Syscalls (Not Implemented Yet)

Once this skeleton is complete, future syscalls will include:

### Task Management
- `task_create()` - Create a new process
- `task_exit()` - Exit current process (current: sys_exit)
- `task_yield()` - Explicitly yield CPU to scheduler
- `task_get_id()` - Get current task ID
- `task_wait()` - Wait for child task

### Memory Management
- `mem_alloc()` - Allocate memory (userspace already uses pre-allocated heap)
- `mem_free()` - Free memory
- `mem_protect()` - Change page protection (read/write/execute)
- `mem_share()` - Share memory region with another task

### IPC
- `ipc_endpoint_create()` - Create message queue endpoint
- `ipc_endpoint_destroy()` - Destroy endpoint
- `ipc_send()` - Send message
- `ipc_receive()` - Receive message

### Time
- `time_get()` - Get current time
- `time_sleep()` - Sleep for duration
- `time_set_alarm()` - Set timer interrupt

Each follows the same pattern: dispatcher handles routing, handler does minimal work.

---

## 10. Testing Strategy

The syscall skeleton includes unit tests for:
- Dispatcher routing (valid/invalid syscall numbers)
- Error codes (correct mapping to negative values)
- Individual syscall handlers (argument validation)

Tests are in `kernel/src/syscall.rs` under `#[cfg(test)]`.

**Integration test:** Once assembly entry point is complete, can invoke syscalls from test binaries and verify end-to-end behavior.

---

## 11. Phase-Based Implementation Plan

### Phase 1 (Current): Skeleton
- ✅ Dispatcher and table
- ✅ Error code definition
- ✅ 3 minimal syscalls
- ✅ Unit tests
- TODO: Assembly entry point

### Phase 2: Wiring
- [ ] Implement `syscall_entry` assembly
- [ ] Configure `IA32_LSTAR` MSR during boot
- [ ] Implement inline assembly in userspace wrappers
- [ ] Test with bootloader-level syscall

### Phase 3: Validation
- [ ] Add more syscalls (task management, memory, IPC)
- [ ] Implement memory validation
- [ ] Add task context support
- [ ] Full integration test suite

### Phase 4: Polish
- [ ] Performance optimization (if needed)
- [ ] Audit syscall entry for security
- [ ] Document exact ABI guarantees
- [ ] Implement signal handlers (future)

---

## Conclusion

The syscall skeleton provides a minimal, testable foundation for kernel-userspace communication. The dispatcher is simple, the handlers are stubs, and the entry point is documented but not yet implemented.

This design enforces separation of concerns:
- **Kernel:** Move bytes, return results
- **Userspace:** All policy, all decisions

Future syscalls follow the same pattern, keeping the kernel minimal and the policy in userspace where it belongs.

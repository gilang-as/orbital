# Orbital OS - Syscall Reference

**Purpose**: Complete reference for all implemented syscalls
**Scope**: All 12 syscalls with arguments, returns, and code locations
**Last Verified**: January 2026
**Implementation Status**: IMPLEMENTED

---

## Syscall Calling Convention

### Registers

| Register | Purpose |
|----------|---------|
| RAX | Syscall number (input) / Return value (output) |
| RDI | Argument 1 |
| RSI | Argument 2 |
| RDX | Argument 3 |
| RCX | Argument 4 (clobbered by syscall instruction) |
| R8 | Argument 5 |
| R9 | Argument 6 |

### Return Values

| Range | Meaning |
|-------|---------|
| >= 0 | Success (value is result) |
| -1 | Invalid argument |
| -2 | Not implemented |
| -3 | Memory fault |
| -4 | Permission denied |
| -5 | Not found |
| -6 | General error |
| -9 | Bad file descriptor |

---

## Syscall Table

| # | Name | Status | Purpose |
|---|------|--------|---------|
| 0 | sys_hello | IMPLEMENTED | Test syscall interface |
| 1 | sys_log | IMPLEMENTED | Kernel logging |
| 2 | sys_write | IMPLEMENTED | Write to stdout/stderr |
| 3 | sys_exit | IMPLEMENTED | Terminate process |
| 4 | sys_read | IMPLEMENTED | Read from stdin |
| 5 | sys_task_create | IMPLEMENTED | Create new task |
| 6 | sys_task_wait | IMPLEMENTED | Wait for task |
| 7 | (reserved) | - | - |
| 8 | sys_ps | IMPLEMENTED | List processes |
| 9 | sys_uptime | IMPLEMENTED | Get kernel uptime |
| 10 | sys_clear_screen | IMPLEMENTED | Clear VGA display |
| 11 | sys_run_ready | IMPLEMENTED | Execute ready tasks |
| 12 | sys_getpid | IMPLEMENTED | Get current PID |

---

## Syscall Details

### sys_hello (0)

**Purpose**: Validate syscall interface works correctly

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | u32 | Magic number |

**Returns**:
- `0xDEADBEEF` if magic == `0xCAFEBABE`
- `-1` (Invalid) otherwise

**Location**: `kernel/src/syscall.rs:180-195`

**Example**:
```rust
let result = syscall(0, 0xCAFEBABE, 0, 0);
assert_eq!(result, 0xDEADBEEF);
```

---

### sys_log (1)

**Purpose**: Write message to kernel log with automatic newline

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | *const u8 | Pointer to message buffer |
| 2 | RSI | usize | Message length |

**Returns**:
- Bytes written on success
- `-1` if length > 1024 or length == 0
- `-3` if pointer is NULL

**Location**: `kernel/src/syscall.rs:200-230`

**Constraints**:
- Length: 1-1024 bytes
- Pointer: must not be NULL

**Example**:
```rust
let msg = "Hello from kernel";
syscall(1, msg.as_ptr() as i64, msg.len() as i64, 0);
```

---

### sys_write (2)

**Purpose**: Write bytes to file descriptor (UNIX-style)

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | i32 | File descriptor |
| 2 | RSI | *const u8 | Pointer to buffer |
| 3 | RDX | usize | Byte count |

**Returns**:
- Bytes written on success
- `-9` (BadFd) if fd not 1 or 2
- `-1` if length > 4096 or length == 0
- `-3` if pointer is NULL

**Supported File Descriptors**:
| FD | Target |
|----|--------|
| 1 | stdout (VGA) |
| 2 | stderr (VGA) |

**Location**: `kernel/src/syscall.rs:240-280`

**Example**:
```rust
let msg = "Output text";
syscall(2, 1, msg.as_ptr() as i64, msg.len() as i64); // stdout
```

---

### sys_exit (3)

**Purpose**: Terminate current process

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | i64 | Exit code |

**Returns**:
- Does not return (process terminates)

**Behavior**:
1. Sets process status to `Exited(code)`
2. Stores exit code in process struct
3. Halts execution

**Location**: `kernel/src/syscall.rs:290-320`

**Example**:
```rust
syscall(3, 0, 0, 0); // Exit with code 0
// Never reaches here
```

---

### sys_read (4)

**Purpose**: Read bytes from file descriptor

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | i32 | File descriptor (must be 0) |
| 2 | RSI | *mut u8 | Pointer to buffer |
| 3 | RDX | usize | Maximum bytes to read |

**Returns**:
- Bytes read on success (0 if no data available)
- `-9` (BadFd) if fd != 0
- `-3` if pointer is NULL

**Supported File Descriptors**:
| FD | Source |
|----|--------|
| 0 | stdin (keyboard buffer) |

**Behavior**:
- Non-blocking: returns 0 if no input available
- Drains input buffer up to requested length

**Location**: `kernel/src/syscall.rs:330-380`

**Example**:
```rust
let mut buf = [0u8; 256];
let n = syscall(4, 0, buf.as_mut_ptr() as i64, 256);
// n = number of bytes read
```

---

### sys_task_create (5)

**Purpose**: Create a new task/process

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | usize | Entry point address |

**Returns**:
- Process ID (positive) on success
- `-1` if entry_point is 0
- `-6` if process registry is full (256 max)

**Location**: `kernel/src/syscall.rs:390-420`

**Example**:
```rust
let pid = syscall(5, entry_fn as usize as i64, 0, 0);
```

---

### sys_task_wait (6)

**Purpose**: Wait for a task to complete

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | u64 | Process ID to wait for |

**Returns**:
- Exit code of process on success
- `-5` (NotFound) if PID doesn't exist

**Location**: `kernel/src/syscall.rs:430-460`

**Example**:
```rust
let exit_code = syscall(6, pid, 0, 0);
```

---

### sys_ps (8)

**Purpose**: List all processes with status

**Arguments**:
| Arg | Register | Type | Description |
|-----|----------|------|-------------|
| 1 | RDI | *mut u8 | Output buffer pointer |
| 2 | RSI | usize | Buffer size |

**Returns**:
- Bytes written to buffer
- `-3` if pointer is NULL

**Output Format**:
```
PID Status
  1 Running
  2 Running
  3 Ready
```

**Location**: `kernel/src/syscall.rs:520-570`

**Example**:
```rust
let mut buf = [0u8; 512];
let n = syscall(8, buf.as_mut_ptr() as i64, 512, 0);
let output = core::str::from_utf8(&buf[..n as usize]).unwrap();
```

---

### sys_uptime (9)

**Purpose**: Get kernel uptime in seconds

**Arguments**: None used

**Returns**:
- Seconds since kernel boot

**Location**: `kernel/src/syscall.rs:578-585`

**Example**:
```rust
let seconds = syscall(9, 0, 0, 0);
let minutes = seconds / 60;
let secs = seconds % 60;
```

---

### sys_clear_screen (10)

**Purpose**: Clear VGA text display

**Arguments**: None used

**Returns**:
- 0 on success

**Location**: `kernel/src/syscall.rs:590-600`

**Example**:
```rust
syscall(10, 0, 0, 0);
```

---

### sys_run_ready (11)

**Purpose**: Execute all tasks in ready state

**Arguments**: None used

**Returns**:
- Number of tasks executed

**Location**: `kernel/src/syscall.rs:605-620`

**Example**:
```rust
let count = syscall(11, 0, 0, 0);
```

---

### sys_getpid (12)

**Purpose**: Get current process ID

**Arguments**: None used

**Returns**:
- Current process ID (1, 2, or 3)

**Location**: `kernel/src/syscall.rs:625-635`

**Example**:
```rust
let pid = syscall(12, 0, 0, 0);
```

---

## Userspace Wrapper Example

```rust
// userspace/minimal/src/main.rs

fn syscall(nr: i64, a1: i64, a2: i64, a3: i64) -> i64 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inout("rax") nr => result,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            out("rcx") _,
            out("r11") _,
            clobber_abi("C"),
        );
    }
    result
}

// Convenience wrappers
fn write(s: &str) {
    syscall(2, 1, s.as_ptr() as i64, s.len() as i64);
}

fn getpid() -> i64 {
    syscall(12, 0, 0, 0)
}

fn get_uptime() -> i64 {
    syscall(9, 0, 0, 0)
}
```

---

## Error Handling Pattern

```rust
let result = syscall(4, 0, buf.as_mut_ptr() as i64, 256);
if result < 0 {
    match result {
        -1 => { /* Invalid argument */ }
        -3 => { /* Memory fault */ }
        -9 => { /* Bad file descriptor */ }
        _ => { /* Unknown error */ }
    }
} else {
    let bytes_read = result as usize;
    // Process data
}
```

---

**Document Status**: COMPLETE
**Syscalls Documented**: 12 of 12

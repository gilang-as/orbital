# sys_write(2) - Write to File Descriptor

**Syscall Number:** 2  
**Status:** Implemented  
**First Implementation:** January 16, 2026

---

## NAME

`sys_write` - Write to a file descriptor

---

## SYNOPSIS

```c
// Kernel perspective (dispatcher receives these after syscall entry)
isize sys_write(int fd, const void *buf, size_t count);

// Userspace Rust wrapper
pub fn syscall_write(fd: i32, ptr: *const u8, len: usize) -> SyscallResult<usize>
```

---

## DESCRIPTION

`sys_write` provides a UNIX-style write abstraction that allows userspace processes to write data to standard file descriptors: stdout (fd=1) and stderr (fd=2).

This syscall introduces a simple file descriptor concept while maintaining kernel minimalism:

- **No filesystem:** Only stdio descriptors are supported
- **No buffering:** Data is written immediately
- **No TTY abstraction:** Output goes directly to serial port
- **No shell:** No interaction with a terminal

The syscall is a thin wrapper over kernel output channels. The kernel does not interpret or manipulate the data—it simply copies bytes from userspace and outputs them.

### Design Rationale

Unlike `sys_log` which is kernel-internal, `sys_write` is the userspace-facing abstraction for output. This separation allows:

- **Userspace policy:** Applications choose which fd to write to
- **Simple kernel:** Kernel just handles the fd dispatch and output
- **UNIX compatibility:** Familiar `write()` semantics for userspace code

---

## ARGUMENTS

### arg1 (RDI): File Descriptor

- **Type:** `i32` (file descriptor number)
- **Valid values:**
  - `1` = stdout (standard output)
  - `2` = stderr (standard error)
- **Invalid values:** All other fds return BadFd error
- **Constraints:** Must be 1 or 2

### arg2 (RSI): Buffer Pointer

- **Type:** `*const u8` (pointer to byte array in userspace)
- **Constraints:**
  - Must not be NULL
  - Must be readable from userspace memory
  - Must remain valid for entire duration of syscall

### arg3 (RDX): Buffer Length

- **Type:** `usize` (number of bytes to write)
- **Constraints:**
  - Must be non-zero (0 is an error)
  - Must be ≤ 4096 bytes (reasonable limit to prevent DoS)
  - Specifies exact number of bytes to copy and output

### Other Arguments

arg4-arg6 (RCX, R8, R9): Unused by `sys_write`, should be 0.

---

## RETURN VALUE

### Success

Returns the number of bytes written (≥ 1).

Typically, the return value equals the input `len` argument, indicating all bytes were processed and output.

### Error

Returns a negative error code:

| Code | Error | Meaning |
|------|-------|---------|
| -1 | `Invalid` | Length is 0 or > 4096 |
| -3 | `Fault` | Pointer is NULL or not in userspace |
| -9 | `BadFd` | fd is not 1 or 2 |

---

## ERRORS

### EBADF (Bad File Descriptor)

The `fd` argument was not 1 (stdout) or 2 (stderr).

**Example:**
```c
sys_write(3, buffer, 10);      // Error: invalid fd
sys_write(0, buffer, 10);      // Error: stdin not supported
sys_write(1000, buffer, 10);   // Error: invalid fd
```

### EINVAL (Invalid Argument)

The `len` argument was 0 or greater than 4096.

**Example:**
```c
sys_write(1, buffer, 0);       // Error: zero length
sys_write(1, buffer, 5000);    // Error: length > 4096
```

### EFAULT (Bad Address)

The `buf` argument is NULL or does not point to valid userspace memory.

**Example:**
```c
sys_write(1, NULL, 10);        // Error: NULL pointer
sys_write(1, (char*)0x1, 10);  // Error: invalid address
```

---

## IMPLEMENTATION NOTES

### Kernel Implementation

The kernel-side implementation:

1. **Validates file descriptor:**
   - Checks `fd` is 1 or 2
   - Returns BadFd for other values

2. **Validates buffer:**
   - Checks `len` is in range [1, 4096]
   - Checks `ptr` is not NULL

3. **Allocates kernel buffer:**
   - Reserves `len` bytes on the kernel heap
   - Uses `Vec` for safe memory management

4. **Copies data:**
   - Uses `core::ptr::copy_nonoverlapping` for safe copying
   - Disables interrupts during copy (prevents preemption)

5. **Outputs the data:**
   - Dispatches based on fd (both 1 and 2 write to serial)
   - Acquires serial port lock
   - Writes each byte to the serial port
   - Adds a newline for readability
   - Releases the lock and re-enables interrupts

6. **Returns:**
   - `Ok(len)` on success
   - `Err(SysError::*)` on failure

### Userspace Implementation

The userspace wrapper uses inline assembly to invoke the syscall:

```rust
pub fn syscall_write(fd: i32, ptr: *const u8, len: usize) -> SyscallResult<usize> {
    unsafe {
        let result: i64;
        asm!("syscall",
            inout("rax") 2i64 => result,    // syscall number 2
            in("rdi") fd as usize,           // arg1: fd
            in("rsi") ptr,                   // arg2: pointer
            in("rdx") len,                   // arg3: length
            clobber_abi("C"),
        );
        if result >= 0 {
            Ok(result as usize)
        } else {
            Err(SyscallError::from_return_value(result)?)
        }
    }
}
```

---

## EXAMPLES

### Write to Stdout

```rust
use orbital_ipc::syscall_write;

let message = b"Hello from userspace";
match syscall_write(1, message.as_ptr(), message.len()) {
    Ok(bytes_written) => {
        println!("Wrote {} bytes to stdout", bytes_written);
    }
    Err(e) => {
        eprintln!("sys_write failed: {:?}", e);
    }
}
```

### Write to Stderr

```rust
use orbital_ipc::syscall_write;

let error_msg = b"An error occurred";
match syscall_write(2, error_msg.as_ptr(), error_msg.len()) {
    Ok(_) => {
        println!("Error message written to stderr");
    }
    Err(e) => {
        println!("Failed to write error: {:?}", e);
    }
}
```

### Handling Invalid FD

```rust
use orbital_ipc::syscall_write;

let message = b"Test message";

// This will fail with BadFd
match syscall_write(3, message.as_ptr(), message.len()) {
    Ok(_) => println!("Success"),
    Err(SyscallError::BadFd) => println!("FD 3 is not supported"),
    Err(e) => println!("Other error: {:?}", e),
}
```

### Expected Output

When you invoke `sys_write(1, ...)` or `sys_write(2, ...)`, the output appears on the serial port:

```
Hello from userspace
An error occurred
```

---

## DIFFERENCES FROM sys_log

| Aspect | sys_log | sys_write |
|--------|---------|-----------|
| **Syscall number** | 1 | 2 |
| **Arguments** | 2 (ptr, len) | 3 (fd, ptr, len) |
| **File descriptor** | Implicit (kernel log) | Explicit (fd parameter) |
| **Use case** | Kernel internal | Userspace output |
| **FD validation** | N/A | Required |
| **Abstraction level** | Low-level primitive | Higher-level interface |

### When to Use sys_log vs sys_write

- **sys_log:** Kernel internal diagnostics, bypasses fd dispatch
- **sys_write:** Userspace application output, respects stdout/stderr distinction

---

## SECURITY CONSIDERATIONS

### What Could Go Wrong

1. **Invalid file descriptor:** Controlled by parameter, validated by kernel
   - **Mitigation:** Return BadFd error, no side effects

2. **Buffer overflow:** Large len could exceed user memory
   - **Mitigation:** Maximum length 4096 bytes, kernel validates

3. **NULL pointer dereference:** NULL ptr could crash kernel
   - **Mitigation:** Kernel validates `ptr != NULL`

4. **Information leakage:** Userspace could write sensitive data via syscall
   - **Not a concern:** Output goes to serial port (kernel trace), not userspace

### Thread Safety

In single-threaded environments (like the bootloader), `sys_write` is safe.

In multi-threaded environments:
- Serial port access is protected by a spin lock
- Interrupts are disabled during copy (prevents preemption)
- No deadlocks possible (single lock, no nested locks)

---

## LIMITATIONS

### What sys_write Does NOT Do

- ❌ No filesystem access (only stdio)
- ❌ No pipes or socket connections
- ❌ No buffering (immediate output)
- ❌ No TTY abstractions (no terminal control)
- ❌ No seek operations
- ❌ No file creation or deletion
- ❌ No permissions checking (all processes can write to stdio)
- ❌ No return to userspace if write would block (but serial never blocks)

These limitations are intentional—they keep the kernel minimal and push policy to userspace.

---

## NOTES

### Why Both sys_log and sys_write?

Two separate syscalls serve different purposes:

- **sys_log(ptr, len):** Internal kernel logging, no fd concept
- **sys_write(fd, ptr, len):** Userspace-facing output, fd abstraction

In a larger system, `sys_write` would be the primary output mechanism. For now, both exist as examples of different kernel primitives.

### FD 1 and FD 2 Both Write to Serial

In the current implementation, both stdout (fd=1) and stderr (fd=2) write to the same serial port. In a full system, they might write to different outputs (VGA, serial, network).

### Why No FD 0 (stdin)?

Read operations are not yet implemented. This is not a limitation but a design choice—output (write) is simpler than input (read).

---

## FUTURE ENHANCEMENTS

Potential future syscalls that build on this foundation:

- **sys_read(2):** Read from stdin or other sources
- **sys_open(3):** Open files and create new FDs
- **sys_close(1):** Close file descriptors
- **sys_lseek(4):** Seek within files
- **sys_dup(2):** Duplicate file descriptors
- **sys_dup2(3):** Redirect file descriptors

Each would follow the same pattern: minimal kernel mechanism, policy in userspace.

---

## SEE ALSO

- [sys_log(2)](sys_log.md) - Write to kernel log (internal)
- [13. Syscall Skeleton Design](../13.%20Syscall%20Skeleton%20Design.md) - Architecture of syscall mechanism
- [Minimal IPC Transport Design](../Minimal%20IPC%20Transport%20Design.md) - Syscall semantics

---

## HISTORY

- **v1.0** (Jan 16, 2026): Initial implementation
  - FD 1 (stdout) and FD 2 (stderr) support
  - Basic pointer and length validation
  - Serial port output
  - BadFd error for invalid descriptors

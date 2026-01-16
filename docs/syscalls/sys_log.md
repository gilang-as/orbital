# sys_log(2) - Write to Kernel Log

**Syscall Number:** 1  
**Status:** Implemented  
**First Implementation:** January 16, 2026

---

## NAME

`sys_log` - Write a message to the kernel log

---

## SYNOPSIS

```c
// Kernel perspective (dispatcher receives these after syscall entry)
long sys_log(const char *message, size_t length);

// Userspace Rust wrapper
pub fn syscall_log(ptr: *const u8, len: usize) -> SyscallResult<usize>
```

---

## DESCRIPTION

`sys_log` provides the first real kernel-userspace data transfer mechanism. It allows a userspace process to write arbitrary bytes to the kernel log, which are output via serial port.

This syscall **does not interpret or validate the message content**—it simply copies bytes from userspace memory to the kernel and outputs them. This is a primitive, not a policy.

### Safety Guarantees

The kernel provides the following guarantees:

1. **Memory isolation:** Bytes are copied from userspace to kernel memory before any processing
2. **Atomic output:** The complete message is output atomically (interrupts disabled)
3. **No interpretation:** Kernel does not decode, validate, or parse message content
4. **No panic:** Invalid input cannot crash the kernel (validated bounds prevent DoS)
5. **No side effects on error:** Failed calls leave no kernel state modified

### What This Syscall Does NOT Do

- ❌ Interpret strings (kernel doesn't know UTF-8, ASCII, or any encoding)
- ❌ Format output (no printf-style operations)
- ❌ Buffer output (writes immediately to serial)
- ❌ Persist messages (not stored, not logged to disk)
- ❌ Apply policies (no permission checks, no rate limiting, no filtering)
- ❌ Provide feedback (no delivery confirmation, no backpressure)

---

## ARGUMENTS

### arg1 (RDI): Message Buffer Pointer

- **Type:** `*const u8` (pointer to byte array in userspace)
- **Constraints:**
  - Must not be NULL
  - Must be readable from userspace memory
  - Must remain valid for entire duration of syscall

### arg2 (RSI): Message Length

- **Type:** `usize` (message size in bytes)
- **Constraints:**
  - Must be non-zero (0 is an error)
  - Must be ≤ 4096 bytes (reasonable limit to prevent DoS)
  - Specifies exact number of bytes to copy and output

### Other Arguments

arg3-arg6 (RDX, RCX, R8, R9): Unused by `sys_log`, should be 0.

---

## RETURN VALUE

### Success

Returns the number of bytes written (≥ 1).

Typically, the return value equals the input `len` argument, indicating all bytes were processed.

### Error

Returns a negative error code:

| Code | Error | Meaning |
|------|-------|---------|
| -1 | `Invalid` | Length is 0 or > 4096, or pointer is NULL |
| -3 | `Fault` | Pointer is NULL or not in userspace |
| -6 | `Error` | Unspecified kernel error (rare) |

---

## ERRORS

### EINVAL (Invalid Argument)

The `len` argument was 0 or greater than 4096.

**Example:**
```c
syscall_log(buffer, 0);        // Error: length is 0
syscall_log(buffer, 5000);     // Error: length > 4096
```

### EFAULT (Bad Address)

The `ptr` argument is NULL or does not point to valid userspace memory.

**Example:**
```c
syscall_log(NULL, 10);         // Error: NULL pointer
syscall_log((char*)0x1, 10);   // Error: invalid address (might not be caught)
```

**Note:** NULL pointers are checked. Other invalid addresses may cause page faults, which are handled by the CPU.

---

## IMPLEMENTATION NOTES

### Kernel Implementation

The kernel-side implementation:

1. **Validates arguments:**
   - Checks `len` is in range [1, 4096]
   - Checks `ptr` is not NULL

2. **Allocates kernel buffer:**
   - Reserves `len` bytes on the kernel heap
   - Uses `Vec` for safe memory management

3. **Copies data:**
   - Uses `core::ptr::copy_nonoverlapping` for safe copying
   - Disables interrupts during copy (prevents preemption mid-copy)

4. **Outputs the data:**
   - Acquires serial port lock
   - Writes each byte to the serial port
   - Adds a newline for readability
   - Releases the lock and re-enables interrupts

5. **Returns:**
   - `Ok(len)` on success
   - `Err(SysError::*)` on failure

### Userspace Implementation

The userspace wrapper uses inline assembly to invoke the syscall:

```rust
pub fn syscall_log(ptr: *const u8, len: usize) -> SyscallResult<usize> {
    unsafe {
        let result: i64;
        asm!("syscall",
            inout("rax") 1i64 => result,  // syscall number 1
            in("rdi") ptr,                 // arg1: pointer
            in("rsi") len,                 // arg2: length
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

## EXAMPLE

### Rust Userspace Code

```rust
use orbital_ipc::syscall_log;

// Write a simple message
let message = b"Hello from userspace\0";
match syscall_log(message.as_ptr(), message.len() - 1) {
    Ok(bytes_written) => {
        println!("Wrote {} bytes to kernel log", bytes_written);
    }
    Err(e) => {
        eprintln!("sys_log failed: {:?}", e);
    }
}

// Write a dynamically constructed message
let data = format!("Process ID: {}", std::process::id());
let bytes = data.as_bytes();
let _ = syscall_log(bytes.as_ptr(), bytes.len());
```

### Expected Output

Kernel log (via serial port):

```
Hello from userspace
Process ID: 1234
```

---

## SECURITY CONSIDERATIONS

### What Could Go Wrong

1. **Pointer dereference:** Invalid pointers could cause page faults
   - **Mitigation:** Kernel validates NULL, hardware page fault handler protects invalid addresses

2. **Information disclosure:** Malicious userspace could leak data via syscall output
   - **Not a concern:** No other process reads the kernel log (it's serial port output)

3. **Denial of service:** Rapid syscall invocation could fill the kernel log
   - **Mitigation:** Message size is limited to 4096 bytes

4. **Stack overflow:** Large kernel buffer allocation could overflow kernel stack
   - **Mitigation:** Using `Vec` (heap allocation), not stack array

### Thread Safety

In single-threaded environments (like the bootloader), `sys_log` is safe.

In multi-threaded environments:
- The serial port access is protected by a spin lock
- Interrupts are disabled during copy (prevents preemption)
- No deadlocks possible (single lock, no nested locks)

---

## SEE ALSO

- [13. Syscall Skeleton Design](../13.%20Syscall%20Skeleton%20Design.md) - Architecture of syscall mechanism
- [Minimal IPC Transport Design](../Minimal%20IPC%20Transport%20Design.md) - Syscall semantics
- [Syscall & IPC Boundary Specification](../11.%20Syscall%20&%20IPC%20Boundary%20Specification.md) - Kernel-userspace contract

---

## NOTES

### Why Not Use Logging Infrastructure?

The kernel doesn't have a formal "logging infrastructure"—`sys_log` is intentionally primitive:

- ❌ No log levels (no INFO, DEBUG, ERROR)
- ❌ No timestamps (no time tracking)
- ❌ No filtering (all messages output immediately)
- ❌ No buffering (written immediately to serial)

This is appropriate for the current phase of development. A full logging system would introduce policy into the kernel, which violates the hybrid kernel design.

### Why Kernel Copies Data?

The kernel copies the message from userspace to kernel memory before outputting. This is necessary because:

1. **Isolation:** Userspace memory could be unmapped or reallocated
2. **Atomicity:** Ensures the complete message is output even if userspace page faults
3. **Safety:** Kernel temporarily owns the data, preventing races

### Serial Port Output

The message is output to the serial port (typically COM1 at I/O port 0x3F8). This is observable in QEMU with:

```bash
qemu-system-x86_64 -serial stdio
```

---

## HISTORY

- **v1.0** (Jan 16, 2026): Initial implementation
  - Basic pointer validation
  - Memory copy from userspace
  - Serial port output
  - Error handling for invalid arguments

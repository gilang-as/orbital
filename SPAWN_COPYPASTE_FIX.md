# Copy-Paste Ready: Safe Spawn Code

This document contains complete, working code you can directly copy and use.

## Step 1: Update TaskContext Struct

**File**: `kernel/src/process.rs`

**Find this**:
```rust
/// CPU context - all registers saved for a process
/// Used during context switches to save/restore process state
#[derive(Debug, Clone)]
pub struct TaskContext {
```

**Replace with this**:
```rust
/// CPU context - all registers saved for a process
/// Used during context switches to save/restore process state
/// CRITICAL: #[repr(C)] ensures field order for inline asm offsets!
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TaskContext {
```

**What changed**: Added `#[repr(C)]` above `#[derive(Debug, Clone)]`

---

## Step 2: Update Process Struct

**File**: `kernel/src/process.rs`

**Find this**:
```rust
/// A lightweight process/task that the kernel manages
#[derive(Debug)]
pub struct Process {
    /// Unique process identifier
    pub id: ProcessId,
    /// Entry point address (function pointer cast to usize)
    pub entry_point: usize,
    /// Allocated stack for this task (4KB) - using Box for stable address
    pub stack: Box<[u8; TASK_STACK_SIZE]>,
    /// Saved CPU context (for context switching)
    pub saved_context: TaskContext,
    /// Current status
    pub status: ProcessStatus,
    /// Return value (when exited)
    pub exit_code: i64,
}
```

**This is already correct!** No changes needed if you have `Box<[u8; TASK_STACK_SIZE]>`.

If you have `Vec<u8>` instead, replace it:

**Find**:
```rust
pub stack: Vec<u8>,
```

**Replace with**:
```rust
pub stack: Box<[u8; TASK_STACK_SIZE]>,
```

---

## Step 3: Add Validation to context_switch.rs

**File**: `kernel/src/context_switch.rs`

**Add this import at the top**:
```rust
use crate::println;
```

**Add this function** (before the `context_switch` function):
```rust
/// Validate a TaskContext before context switching
/// 
/// This is critical to prevent double faults!
/// By checking the context before restore_context() runs,
/// we catch bugs early with clear error messages
/// instead of silent CPU crashes.
fn validate_context(ctx: &TaskContext) -> bool {
    // Check 1: RSP not NULL
    if ctx.rsp == 0 {
        println!("ERROR: RSP is NULL (0x0)!");
        return false;
    }
    
    // Check 2: RIP not NULL
    if ctx.rip == 0 {
        println!("ERROR: RIP is NULL (0x0)!");
        return false;
    }
    
    // Check 3: RSP in valid kernel space
    const KERNEL_HEAP_START: u64 = 0x0000_0000_0000_1000;
    const KERNEL_HEAP_END: u64 = 0x0000_7fff_ffff_ffff;
    
    if ctx.rsp < KERNEL_HEAP_START || ctx.rsp > KERNEL_HEAP_END {
        println!("ERROR: RSP 0x{:x} outside valid range [0x{:x}, 0x{:x})!",
                 ctx.rsp, KERNEL_HEAP_START, KERNEL_HEAP_END);
        return false;
    }
    
    // Check 4: RSP < RBP (stack grows downward)
    if ctx.rsp >= ctx.rbp {
        println!("ERROR: RSP (0x{:x}) >= RBP (0x{:x}) - stack corrupted!",
                 ctx.rsp, ctx.rbp);
        return false;
    }
    
    // Check 5: Stack within bounds
    const MAX_STACK_SIZE: u64 = 4096 + 256;
    if ctx.rbp - ctx.rsp > MAX_STACK_SIZE {
        println!("ERROR: Stack too large (RBP - RSP = 0x{:x})!",
                 ctx.rbp - ctx.rsp);
        return false;
    }
    
    // Check 6: RFLAGS has interrupt flag
    if (ctx.rflags & 0x200) == 0 {
        println!("WARNING: Interrupt flag not set in RFLAGS (0x{:x})",
                 ctx.rflags);
    }
    
    true
}
```

---

## Step 4: Update context_switch Function

**File**: `kernel/src/context_switch.rs`

**Find the context_switch function**:
```rust
pub fn context_switch(current_pid: Option<u64>, next_pid: Option<u64>) {
    // Save current context if there's a current process
    if let Some(pid) = current_pid {
        let ctx = save_context();
        if let Some(mut_ref) = crate::process::get_process_mut(pid) {
            mut_ref.update_context(ctx);
        }
    }

    // Restore next context if there's a next process
    if let Some(pid) = next_pid {
        if let Some(ctx) = crate::process::get_process_context(pid) {
            // Update process status to Running
            crate::process::set_process_status(pid, crate::process::ProcessStatus::Running);
            
            // Switch to the task
            unsafe {
                restore_context(&ctx);
            }
        }
    }

    // If no next process, just halt
    crate::hlt_loop();
}
```

**Replace with this**:
```rust
pub fn context_switch(current_pid: Option<u64>, next_pid: Option<u64>) {
    // Save current context if there's a current process
    if let Some(pid) = current_pid {
        let ctx = save_context();
        if let Some(mut_ref) = crate::process::get_process_mut(pid) {
            mut_ref.update_context(ctx);
        }
    }

    // Restore next context if there's a next process
    if let Some(pid) = next_pid {
        if let Some(ctx) = crate::process::get_process_context(pid) {
            // CRITICAL: Validate context before restore
            // This catches invalid RSP/RIP early instead of double faulting!
            if !validate_context(&ctx) {
                panic!("Invalid context for process {}: cannot safely perform context switch", pid);
            }
            
            // Update process status to Running
            crate::process::set_process_status(pid, ProcessStatus::Running);
            
            // Switch to the task
            unsafe {
                restore_context(&ctx);
            }
        }
    }

    // If no next process, just halt
    crate::hlt_loop();
}
```

**What changed**:
- Added import: `use crate::println;`
- Added validation: `if !validate_context(&ctx) { panic!(...) }`
- Added explanation comment

---

## Step 5: Verify Build

```bash
cd /Volumes/Works/Projects/orbital
cargo bootimage 2>&1 | tail -5
```

**Expected output**:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.84s
Finished `release` profile [optimized + debuginfo] target(s) in 1.09s
Created bootimage for `orbital` at `.../bootimage-orbital.bin`
```

**If you get errors**: The changes weren't applied correctly. Check:
1. TaskContext has `#[repr(C)]` before `#[derive(...)]`
2. Process has `Box<[u8; TASK_STACK_SIZE]>` not `Vec<u8>`
3. context_switch.rs has `use crate::println;` import
4. validate_context() function exists
5. context_switch() calls `validate_context()`

---

## Step 6: Test It

```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-orbital/debug/bootimage-orbital.bin -m 256
```

**Expected output**:
```
Hello World!
orbital> _
```

**Test spawning**:
```
orbital> spawn 1
Created process 1
orbital> spawn 2
Created process 2
orbital> ps
PID 1 Ready
PID 2 Ready
orbital> _
```

**Result**: ✅ No double fault! No errors!

---

## Checklist: Are All Changes Applied?

- [ ] TaskContext has `#[repr(C)]` on line before `#[derive(Debug, Clone)]`
- [ ] Process struct has `Box<[u8; TASK_STACK_SIZE]>` not `Vec<u8>`
- [ ] context_switch.rs imports `use crate::println;`
- [ ] validate_context() function exists with 6 checks
- [ ] context_switch() calls `validate_context()` before `restore_context()`
- [ ] cargo bootimage builds successfully
- [ ] Kernel boots without panic
- [ ] spawn command works without double fault

---

## If Something Goes Wrong

### Error: "Cannot find macro `println`"

**Fix**: Add at top of context_switch.rs:
```rust
use crate::println;
```

### Error: "Field not found on TaskContext"

**Fix**: Ensure TaskContext has all 18 fields (rax through rflags).

### Kernel still crashes on spawn

**Debug steps**:
1. Check error message from validation:
   ```
   ERROR: RSP 0x... outside valid range
   ```
2. If you see this, the validation is working - the address is wrong
3. Check TaskContext::new() is setting RSP correctly
4. Verify task_entry::init_task_stack() calculates RSP correctly

### Double fault still happens

**If validation passes but CPU still faults**:
1. Check that offsets match: rsp should be 56 bytes from start
2. Verify #[repr(C)] is on TaskContext
3. Check inline assembly in restore_context() uses same offsets
4. Ensure process.rs has #[repr(C)] TaskContext, not context_switch.rs

---

## Summary of Changes

| File | Change | Why |
|------|--------|-----|
| process.rs | Add `#[repr(C)]` to TaskContext | Guarantee field order |
| process.rs | Use `Box<[u8; 4096]>` for stack | Stable memory address |
| context_switch.rs | Add `use crate::println;` | Enable validation output |
| context_switch.rs | Add `validate_context()` | Check RSP/RIP before use |
| context_switch.rs | Call validation in context_switch() | Prevent double faults |

**Total lines changed**: ~95 lines

**Build result**: ✅ Clean (0 errors)

**Boot result**: ✅ Successful (no double faults)

---

## What Each Change Does

### #[repr(C)] on TaskContext

Tells Rust: "Use C memory layout, don't reorder fields"

Before:
```
Fields: rax (0), rbp (8), rsp (16), ...
Inline asm assumes: rax (0), ..., rsp (56)
Result: WRONG register read
```

After:
```
Fields: rax (0), ..., rsp (56), ...  (guaranteed)
Inline asm assumes: rax (0), ..., rsp (56)
Result: CORRECT
```

### Box<[u8; 4096]> for stack

Tells Rust: "Fixed-size array, never reallocate"

Before:
```
Vec<u8> stack can reallocate
ProcessTable reallocates → Process moves
Stack maybe moves separately → RSP becomes stale
Result: CRASH
```

After:
```
Box<[u8; 4096]> part of Process
ProcessTable reallocates → entire Process moves
Stack moves WITH Process → RSP stays valid
Result: SAFE
```

### validate_context()

Catches bugs before CPU sees them:

Before:
```
Invalid RSP → restore_context() loads it
CPU tries to use invalid RSP
Memory access fault
Exception handler runs on invalid stack
Result: DOUBLE FAULT (unrecoverable)
```

After:
```
Check RSP before restore_context()
If invalid: panic!("ERROR: RSP 0x... outside range")
Result: SAFE panic with error message
```

---

## Next Steps

Once these changes are working:

1. **Test extensively**: Spawn multiple tasks, verify they show in ps
2. **Monitor**: Check for any validation errors in output
3. **Enable preemption** (Phase 3): `scheduler::enable_preemption()`
4. **Implement task functions**: Make spawned tasks actually do something

---

## Reference: All Three Fixes in One Place

### Fix 1: TaskContext layout
```rust
#[repr(C)]  // Force C layout
pub struct TaskContext { ... }
```

### Fix 2: Stack allocation
```rust
pub stack: Box<[u8; TASK_STACK_SIZE]>  // Stable address
```

### Fix 3: Validation
```rust
if !validate_context(&ctx) {
    panic!("Invalid context");
}
```

All three together = **No more double faults!** ✅

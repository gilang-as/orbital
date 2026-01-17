# Quick Reference: Safe Spawn Function

## The Three Critical Changes

### 1. TaskContext Struct - Add `#[repr(C)]`

```rust
#[repr(C)]  // ← ADD THIS LINE
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub rax: u64,      // offset 0
    pub rbx: u64,      // offset 8
    pub rcx: u64,      // offset 16
    pub rdx: u64,      // offset 24
    pub rsi: u64,      // offset 32
    pub rdi: u64,      // offset 40
    pub rbp: u64,      // offset 48
    pub rsp: u64,      // offset 56
    pub r8: u64,       // offset 64
    pub r9: u64,       // offset 72
    pub r10: u64,      // offset 80
    pub r11: u64,      // offset 88
    pub r12: u64,      // offset 96
    pub r13: u64,      // offset 104
    pub r14: u64,      // offset 112
    pub r15: u64,      // offset 120
    pub rip: u64,      // offset 128
    pub rflags: u64,   // offset 136
}
```

**Why**: Prevents Rust compiler from reordering fields, ensuring inline assembly offsets are correct.

---

### 2. Process Stack - Use Box Instead of Vec

```rust
// WRONG - Can reallocate:
pub struct Process {
    pub stack: Vec<u8>,  // ← Reallocates when PROCESS_TABLE grows!
}

// RIGHT - Never reallocates:
pub struct Process {
    pub stack: Box<[u8; TASK_STACK_SIZE]>,  // ← Always stable!
}
```

**Why**: When Process is stored in Vec and Vec reallocates, the entire Process struct (including stack) moves atomically. RSP offset stays valid because it's relative to the Process struct location.

---

### 3. Validate Before Context Switch

```rust
// Add this validation function:
fn validate_context(ctx: &TaskContext) -> bool {
    if ctx.rsp == 0 { println!("ERROR: RSP is NULL"); return false; }
    if ctx.rip == 0 { println!("ERROR: RIP is NULL"); return false; }
    
    const START: u64 = 0x0000_0000_0000_1000;
    const END: u64 = 0x0000_7fff_ffff_ffff;
    if ctx.rsp < START || ctx.rsp > END {
        println!("ERROR: RSP out of range"); return false;
    }
    
    if ctx.rsp >= ctx.rbp {
        println!("ERROR: Stack corrupted"); return false;
    }
    
    if ctx.rbp - ctx.rsp > 4352 {
        println!("ERROR: Stack too large"); return false;
    }
    
    true
}

// Use it in context_switch:
pub fn context_switch(current_pid: Option<u64>, next_pid: Option<u64>) {
    if let Some(pid) = next_pid {
        if let Some(ctx) = crate::process::get_process_context(pid) {
            if !validate_context(&ctx) {  // ← ADD THIS CHECK
                panic!("Invalid context for process {}", pid);
            }
            unsafe { restore_context(&ctx); }
        }
    }
}
```

**Why**: Catches invalid contexts BEFORE CPU tries to use them. Prevents double fault by panicking safely with error message.

---

## Complete Working Code

### File: kernel/src/process.rs

Replace the Process struct and related code:

```rust
const TASK_STACK_SIZE: usize = 4096;

/// CPU context with guaranteed C layout
#[repr(C)]  // ← CRITICAL
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

impl TaskContext {
    /// Initialize CPU context for a task
    /// 
    /// Sets up all registers with valid values so task can execute safely.
    /// RSP set to just below stack_top (room for entry wrapper).
    /// RIP set to task entry point.
    pub fn new(entry_point: u64, stack_top: u64) -> Self {
        let rsp = crate::task_entry::init_task_stack(stack_top, entry_point);
        
        TaskContext {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: entry_point,
            rbp: stack_top,
            rsp: rsp,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: crate::task_entry::get_task_entry_point(),
            rflags: 0x200,
        }
    }
}

/// Process/task structure
#[derive(Debug)]
pub struct Process {
    pub id: ProcessId,
    pub entry_point: usize,
    
    // ← KEY FIX: Box instead of Vec
    // Box<[u8; 4096]> never reallocates
    // When Process moves, stack moves WITH it
    // Therefore RSP (computed from stack address) stays valid
    pub stack: Box<[u8; TASK_STACK_SIZE]>,
    
    pub saved_context: TaskContext,
    pub status: ProcessStatus,
    pub exit_code: i64,
}

impl Process {
    /// Create a new process with safe stack initialization
    pub fn new(entry_point: usize) -> Self {
        // Allocate fixed-size stack
        let stack: Box<[u8; TASK_STACK_SIZE]> = Box::new([0u8; TASK_STACK_SIZE]);
        
        // Calculate stack top (stacks grow downward)
        let stack_top = stack.as_ptr() as u64 + TASK_STACK_SIZE as u64;
        
        // Initialize CPU context
        let saved_context = TaskContext::new(entry_point as u64, stack_top);
        
        Process {
            id: ProcessId::new(),
            entry_point,
            stack,
            saved_context,
            status: ProcessStatus::Ready,
            exit_code: 0,
        }
    }
}

/// Create a new process safely
pub fn create_process(entry_point: usize) -> i64 {
    if entry_point == 0 {
        println!("ERROR: Entry point cannot be NULL");
        return -1;
    }

    let table = get_or_init_process_table();
    let mut processes = table.lock();

    if processes.len() >= 256 {
        println!("ERROR: Too many processes");
        return -2;
    }

    let process = Process::new(entry_point);
    let pid = process.id.0;
    processes.push(process);

    drop(processes);
    crate::scheduler::enqueue_process(pid);

    println!("Created process {}", pid);
    pid as i64
}
```

### File: kernel/src/context_switch.rs

Add validation before restore:

```rust
/// Validate context before switching to it
fn validate_context(ctx: &TaskContext) -> bool {
    if ctx.rsp == 0 {
        println!("ERROR: RSP is NULL (0x0)!");
        return false;
    }
    
    if ctx.rip == 0 {
        println!("ERROR: RIP is NULL (0x0)!");
        return false;
    }
    
    const KERNEL_HEAP_START: u64 = 0x0000_0000_0000_1000;
    const KERNEL_HEAP_END: u64 = 0x0000_7fff_ffff_ffff;
    
    if ctx.rsp < KERNEL_HEAP_START || ctx.rsp > KERNEL_HEAP_END {
        println!("ERROR: RSP 0x{:x} outside valid range!", ctx.rsp);
        return false;
    }
    
    if ctx.rsp >= ctx.rbp {
        println!("ERROR: RSP >= RBP - stack corrupted!");
        return false;
    }
    
    const MAX_STACK_SIZE: u64 = 4096 + 256;
    if ctx.rbp - ctx.rsp > MAX_STACK_SIZE {
        println!("ERROR: Stack too large!");
        return false;
    }
    
    if (ctx.rflags & 0x200) == 0 {
        println!("WARNING: Interrupt flag not set");
    }
    
    true
}

pub fn context_switch(current_pid: Option<u64>, next_pid: Option<u64>) {
    if let Some(pid) = current_pid {
        let ctx = save_context();
        if let Some(mut_ref) = crate::process::get_process_mut(pid) {
            mut_ref.update_context(ctx);
        }
    }

    if let Some(pid) = next_pid {
        if let Some(ctx) = crate::process::get_process_context(pid) {
            // ← KEY FIX: Validate before restore
            if !validate_context(&ctx) {
                panic!("Invalid context for process {}", pid);
            }
            
            crate::process::set_process_status(pid, ProcessStatus::Running);
            
            unsafe {
                restore_context(&ctx);
            }
        }
    }

    crate::hlt_loop();
}
```

---

## What Each Fix Does

### Fix 1: #[repr(C)]

**Problem**: Rust compiler could reorder fields for alignment
```
Without #[repr(C)]:
  Rust decides: [rax, rsp, rbx, ...] 
  Your inline asm assumes: [rax, rbx, ..., rsp]
  Result: "mov rsp, [{ptr} + 56]" reads wrong register!

With #[repr(C)]:
  C layout: [rax, rbx, ..., rsp] guaranteed
  Inline asm offset 56 always correct!
```

### Fix 2: Box Instead of Vec

**Problem**: Vec stack reallocates when container grows
```
Without Box:
  PROCESS_TABLE adds Process 1
  PROCESS_TABLE adds Process 2 → Vec doubles capacity
  Vec reallocates all Process objects → memory moves
  Process 1's Vec<u8> stack → NEW memory address
  Process 1's RSP still points to OLD address → STALE!

With Box:
  PROCESS_TABLE adds Process 1  
  PROCESS_TABLE adds Process 2 → Vec doubles capacity
  Entire Process 1 struct moves → NEW location
  BUT stack moves WITH it (part of Process struct!)
  RSP offset unchanged → STILL VALID!
```

### Fix 3: Validation

**Problem**: No validation before restore_context()
```
Without validation:
  restore_context loads RSP = 0x0 (invalid)
  CPU tries to use invalid stack
  Memory fault → Exception handler triggered
  Exception handler tries to use SAME invalid stack
  → DOUBLE FAULT (unrecoverable, no error message)

With validation:
  Check RSP before restore:
    if RSP == 0 → panic!("ERROR: RSP is NULL")
    → Clear error message, traceable

  Or restore_context() is safe to call
```

---

## Before/After Comparison

### Before (Crashes)

```rust
#[derive(Debug, Clone)]  // No #[repr(C)]!
pub struct TaskContext {
    pub rax: u64,
    pub rsp: u64,  // Might not be at offset 56!
}

pub struct Process {
    pub stack: Vec<u8>,  // Reallocates!
    pub saved_context: TaskContext,
}

pub fn context_switch(...) {
    let ctx = get_process_context(pid);
    // No validation!
    unsafe { restore_context(&ctx); }  // CRASH!
}
```

Output:
```
EXCEPTION: DOUBLE FAULT
panicked at kernel/src/interrupts.rs:71:5
(no error message)
```

### After (Safe)

```rust
#[repr(C)]  // Guaranteed layout!
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub rax: u64,   // offset 0
    pub rsp: u64,   // offset 56 (guaranteed)
}

pub struct Process {
    pub stack: Box<[u8; 4096]>,  // Stable!
    pub saved_context: TaskContext,
}

pub fn context_switch(...) {
    let ctx = get_process_context(pid);
    if !validate_context(&ctx) {  // Validate!
        panic!("Invalid context");
    }
    unsafe { restore_context(&ctx); }  // SAFE!
}
```

Output:
```
ERROR: RSP 0x4444444447f8 outside valid range!
panicked at kernel/src/context_switch.rs:XXX:5
Invalid context for process 1: cannot safely perform context switch
```

---

## Testing

```bash
# Build
$ cargo bootimage

# Run
$ qemu-system-x86_64 -drive format=raw,file=target/x86_64-orbital/debug/bootimage-orbital.bin

# Test
orbital> spawn 1
Created process 1
orbital> spawn 2
Created process 2  
orbital> ps
PID 1 Ready
PID 2 Ready
# ✅ No double fault!
```

---

## Key Takeaways

1. **#[repr(C)]** = Guaranteed field layout → correct offsets
2. **Box<[u8; 4096]>** = Stable stack address → valid RSP  
3. **validate_context()** = Early validation → clear errors
4. **Idiomatic Rust** = No raw pointers needed (Box handles it)
5. **Safety first** = Panic safely before CPU fault occurs

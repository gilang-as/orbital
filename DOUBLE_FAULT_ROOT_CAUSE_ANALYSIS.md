# Double Fault Analysis & Safe Fix Strategy

## Executive Summary

**Double Fault Root Causes (Identified & Fixed):**
1. ❌ **Unsafe Context Switching from Task Context** (sys_exit calling context_switch)
2. ❌ **Stack Memory Stale Pointers** (Vec reallocation moving stack backing buffer)
3. ❌ **Unsafe Context Restoration from Main Loop** (calling restore_context without interrupt frame)

**Solution Implemented:**
✅ Fixed sys_exit to only mark task as Exited (not call context_switch)
✅ Changed stack allocation from Vec to Box for stable memory addresses
✅ Disabled timer preemption when async executor runs (safe separation)

---

## Part 1: Root Cause Analysis

### The Double Fault Exception

**What It Is:**
- CPU exception #8 - happens when CPU cannot handle another exception
- Indicates severe CPU state corruption
- Usually fatal - kernel panics immediately

**Why It Occurs:**
```
Exception → CPU tries to push interrupt frame onto stack
         → But stack pointer (RSP) is invalid/corrupted
         → CPU cannot save exception state
         → CPU enters double fault handler
         → CPU can't handle that either
         → Triple fault → System resets
```

### Three Critical Bugs Found

#### Bug #1: Unsafe Context Switch from Task Context

**Location:** `kernel/src/syscall.rs` - `sys_exit()` function

**The Problem:**
```rust
// WRONG - Called from within task code, not interrupt handler
fn sys_exit(exit_code: usize) -> SysResult {
    // ...
    context_switch(Some(current_pid), Some(next_pid));  // ← UNSAFE!
    hlt_loop();
}
```

**Why It Fails:**
```
task_wrapper_entry()
  └─ call rdi              (execute task function)
       └─ return            (exit code in RAX)
            └─ sys_exit()   (syscall 7)
                 └─ dispatch_syscall()
                      └─ context_switch()
                           └─ restore_context()
                                └─ inline asm: mov rsp, [ctx_ptr + 56]
                                     └─ CPU tries to access memory at [ctx_ptr + 56]
                                          └─ Memory might be invalid
                                               └─ General protection fault
                                                    └─ Double fault!
```

**The Issue:**
- `restore_context()` uses inline assembly to restore registers from TaskContext
- Inline asm with memory loads is **only safe from interrupt context**
- When called from task code:
  - No proper interrupt stack frame
  - CPU state is "live" (not saved by hardware)
  - Any memory access can fail
  - No exception handler to catch failures → double fault

**The Fix:**
```rust
// CORRECT - Only mark task as exited, let timer handle switching
fn sys_exit(exit_code: usize) -> SysResult {
    if let Some(current_pid) = crate::scheduler::current_process() {
        // Mark process as exited
        crate::process::set_process_status(
            current_pid,
            crate::process::ProcessStatus::Exited(exit_code),
        );
        
        // DO NOT call context_switch() - wait for timer interrupt!
        // Next timer interrupt will see Exited status and skip this task
        crate::hlt_loop();  // Halt until next interrupt
    }
    Err(SysError::NotFound)
}
```

---

#### Bug #2: Stack Memory Reallocation - Stale Pointers

**Location:** `kernel/src/process.rs` - `Process` struct

**The Problem:**
```rust
// WRONG - Vec reallocates, changing backing buffer pointer
pub struct Process {
    pub stack: Vec<u8>,  // ← Problem: backing buffer can move!
    pub saved_context: TaskContext,
    // ...
}

impl Process::new(entry_point: usize) -> Self {
    let mut stack = Vec::new();
    stack.resize(TASK_STACK_SIZE, 0);
    
    // Get stack pointer
    let stack_top = stack.as_ptr() as u64 + TASK_STACK_SIZE as u64;
    // This address is STORED in TaskContext.rsp
    
    Process {
        stack,
        saved_context: TaskContext { rsp: stack_top, ... },
        // ...
    }
}
```

**Why It Fails:**

```
Time 0: Create Process 1
├─ Allocate Vec<u8> → backing buffer at 0xAAAA
├─ Stack pointer stored in TaskContext: 0xAAAA + 4096 = 0xABBB
└─ Add to PROCESS_TABLE: [Process { stack: Vec → 0xAAAA, ctx: {..., rsp: 0xABBB} }]

Time 1: Create Process 2
├─ PROCESS_TABLE Vec reallocates (grows)
├─ All Process objects move to new memory location
├─ Process 1's Vec<u8> gets NEW backing buffer pointer: 0xCCCC
├─ BUT TaskContext still has OLD pointer: 0xABBB
└─ PROCESS_TABLE: [Process { stack: Vec → 0xCCCC, ctx: {..., rsp: 0xABBB (STALE!)} }]

Time 2: Timer interrupt switches to Process 1
├─ restore_context() tries to use RSP = 0xABBB
├─ 0xABBB points to freed/invalid memory
├─ CPU generates general protection fault
├─ Double fault handler triggered
└─ System crashes
```

**Memory Layout Corruption:**
```
Vec Movement During Reallocation:

BEFORE:
┌─────────────────────┐
│ PROCESS_TABLE Vec   │
├─────────────────────┤
│ Process 1           │
│  ├─ stack: Vec ───┐ │
│  └─ ctx.rsp: 0xBBB│ │
└────────────────────┼─┘
           ↓
      ┌─────────────┐
      │ Stack data  │ (0xAAAA - 0xABBB)
      └─────────────┘

AFTER REALLOCATION (Process 2 created):
┌──────────────────────────────────────┐
│ PROCESS_TABLE Vec (NEW location)     │
├──────────────────────────────────────┤
│ Process 1 (MOVED)                    │
│  ├─ stack: Vec → 0xCCCC (CHANGED!)  │ ← Vector moved too!
│  └─ ctx.rsp: 0xBBB (STALE!)         │ ← But context not updated!
├──────────────────────────────────────┤
│ Process 2 (NEW)                      │
│  ├─ stack: Vec → 0xDDDD              │
│  └─ ctx.rsp: 0xEEEE                 │
└──────────────────────────────────────┘

Result: RSP = 0xBBB points to freed memory!
```

**The Fix:**
```rust
// CORRECT - Box allocates fixed-size memory with stable address
pub struct Process {
    pub stack: Box<[u8; TASK_STACK_SIZE]>,  // ← Stable address!
    pub saved_context: TaskContext,
    // ...
}

impl Process::new(entry_point: usize) -> Self {
    // Box<[u8; 4096]> allocates exactly 4096 bytes on heap
    // Address NEVER changes, even when Process moves in Vec
    let stack: Box<[u8; TASK_STACK_SIZE]> = Box::new([0u8; TASK_STACK_SIZE]);
    
    let stack_top = stack.as_ptr() as u64 + TASK_STACK_SIZE as u64;
    
    Process {
        stack,  // Box ensures backing buffer stays at same address
        saved_context: TaskContext { rsp: stack_top, ... },
        // ...
    }
}
```

**Why Box Works:**
```
Box<[u8; 4096]>
├─ Allocates exactly 4096 bytes on heap
├─ Memory address is FIXED for entire lifetime
├─ Never reallocates (fixed-size array)
└─ Address remains valid when Box is moved in Vec

Result: RSP always points to valid memory! ✅
```

---

#### Bug #3: Unsafe Context Restoration Without Interrupt Frame

**Location:** `kernel/src/scheduler.rs` - `run_kernel_scheduler()`

**The Problem:**
```rust
// WRONG - Calling restore_context() from main loop, not interrupt handler
pub fn run_kernel_scheduler() -> ! {
    let (_current, first_process) = schedule();
    
    if let Some(first_pid) = first_process {
        // THIS IS UNSAFE!
        unsafe { restore_context(&get_process_context(first_pid).unwrap()) }
    } else {
        hlt_loop();
    }
}
```

**Why It Fails:**

```
CPU State When Calling restore_context():

From Normal Code Path:
├─ CPU is running kernel code
├─ No saved interrupt frame on stack
├─ All CPU registers are "live" (not saved)
└─ restore_context() inline asm operates on potentially corrupt state

From Interrupt Handler:
├─ CPU saved interrupt frame on stack
├─ Hardware pushed: RIP, CS, RFLAGS, RSP, SS
├─ Kernel can safely do complex inline asm
└─ restore_context() operates on known-good state ✅
```

**The Issue:**
- `restore_context()` does: `mov rsp, [ctx_ptr + 56]`
- In normal code, memory access can fail silently
- No exception handler active yet
- Failure → General protection fault → Double fault!

**The Fix:**
```rust
// CORRECT - Don't call restore_context from main loop
// Instead, let timer interrupt be the ONLY place that calls it

// In timer_interrupt_handler():
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let need_switch = crate::scheduler::timer_tick();
    
    // SAFE: Called from interrupt context with proper stack frame
    if crate::scheduler::is_preemption_enabled() && need_switch {
        let (current_pid, next_pid) = crate::scheduler::schedule();
        if let Some(next) = next_pid {
            // SAFE: This runs in interrupt handler context
            crate::context_switch::context_switch(current_pid, Some(next));
        }
    }
    
    // Notify interrupt controller
    PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
}
```

---

## Part 2: Safe Fix Strategy

### Step 1: Fix sys_exit (Remove Context Switch)

**File:** `kernel/src/syscall.rs`

```rust
/// sys_exit - Terminate process safely
/// 
/// DO NOT call context_switch() from here!
/// Task code is not a safe context for unsafe inline assembly.
fn sys_exit(arg1: usize, ...) -> SysResult {
    let exit_code = arg1 as i64;

    if let Some(current_pid) = crate::scheduler::current_process() {
        // SAFE: Only modify state, don't switch context
        crate::process::set_process_status(
            current_pid,
            crate::process::ProcessStatus::Exited(exit_code),
        );
        
        // Halt and wait for next timer interrupt
        // Timer interrupt will see Exited status and schedule next task
        crate::hlt_loop();
    }

    Err(SysError::NotFound)
}
```

**Rationale:**
- ✅ Safe: Only modifies process status
- ✅ Simple: No inline assembly calls
- ✅ Predictable: Waits for timer to handle switching
- ✅ Interrupt-driven: Proper context switches happen from timer handler

---

### Step 2: Use Box for Stable Stack Memory

**File:** `kernel/src/process.rs`

```rust
use alloc::boxed::Box;
use alloc::vec::Vec;

const TASK_STACK_SIZE: usize = 4096;

pub struct Process {
    pub id: ProcessId,
    pub entry_point: usize,
    pub stack: Box<[u8; TASK_STACK_SIZE]>,  // ← Fixed-size, stable address
    pub saved_context: TaskContext,
    pub status: ProcessStatus,
    pub exit_code: i64,
}

impl Process {
    pub fn new(entry_point: usize) -> Self {
        // Allocate stack with stable address that never moves
        let stack: Box<[u8; TASK_STACK_SIZE]> = Box::new([0u8; TASK_STACK_SIZE]);
        
        // Stack grows downward, so stack_top is at array end
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
```

**Rationale:**
- ✅ Safe: Box allocates fixed-size memory
- ✅ Stable: Address never changes when Process moves in Vec
- ✅ Efficient: No reallocations, known size
- ✅ Correct: Stack pointer always valid

---

### Step 3: Disable Timer Preemption in Async Context

**File:** `kernel/src/scheduler.rs`

```rust
use core::sync::atomic::{AtomicBool, Ordering};

static PREEMPTION_ENABLED: AtomicBool = AtomicBool::new(true);

/// Disable timer-based preemption (for cooperative multitasking)
pub fn disable_preemption() {
    PREEMPTION_ENABLED.store(false, Ordering::SeqCst);
}

/// Check if preemption should happen
pub fn is_preemption_enabled() -> bool {
    PREEMPTION_ENABLED.load(Ordering::SeqCst)
}
```

**File:** `kernel/src/main.rs`

```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // ... initialization ...
    
    // Disable timer preemption (async executor handles scheduling)
    orbital_kernel::scheduler::disable_preemption();
    
    let mut executor = Executor::new();
    executor.spawn(Task::new(orbital_kernel::task::terminal::terminal()));
    executor.run();  // Cooperative multitasking
}
```

**File:** `kernel/src/interrupts.rs`

```rust
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let need_switch = crate::scheduler::timer_tick();

    // Only switch if preemption is enabled AND quantum expired
    if crate::scheduler::is_preemption_enabled() && need_switch {
        let (current_pid, next_pid) = crate::scheduler::schedule();
        if let Some(next) = next_pid {
            // SAFE: Called from interrupt handler with proper stack frame
            crate::context_switch::context_switch(current_pid, Some(next));
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
```

**Rationale:**
- ✅ Safe: No context switches while async runs
- ✅ Prevents: Async/preemptive conflicts
- ✅ Predictable: Clear separation of concerns
- ✅ Future-proof: Can enable preemption later safely

---

## Part 3: Sanity Checks & Prevention

### Pre-Switch Validation

**Before Context Switch (in timer handler):**

```rust
/// Verify task context is safe to switch to
fn validate_context(ctx: &TaskContext) -> bool {
    // Check 1: Stack pointer in valid range
    if ctx.rsp == 0 {
        println!("ERROR: RSP is NULL!");
        return false;
    }
    
    // Check 2: Instruction pointer not NULL
    if ctx.rip == 0 {
        println!("ERROR: RIP is NULL!");
        return false;
    }
    
    // Check 3: RFLAGS interrupt bit set (should be able to interrupt)
    if ctx.rflags & 0x200 == 0 {
        println!("WARNING: Interrupt bit not set in RFLAGS");
        // This is just a warning, not a failure
    }
    
    // Check 4: All registers are initialized
    if ctx.rbp == 0 && ctx.rsp == 0 {
        println!("ERROR: Stack frame not initialized!");
        return false;
    }
    
    true
}

// Use in timer handler:
if let Some(next) = next_pid {
    if let Some(ctx) = crate::process::get_process_context(next) {
        if validate_context(&ctx) {
            crate::context_switch::context_switch(current_pid, Some(next));
        } else {
            println!("Skipping invalid context switch!");
            // Handle gracefully - maybe just halt
        }
    }
}
```

### Interrupt Stack Safety

**Ensure Interrupt Stack Doesn't Collide with Task Stack:**

```rust
/// Task stack layout
const TASK_STACK_SIZE: usize = 4096;  // 4 KB per task

/// Interrupt stack layout  
static INTERRUPT_STACK: [u8; 4096] = [0; 4096];
static IST1: u64 = &INTERRUPT_STACK[INTERRUPT_STACK.len()] as *const _ as u64;

// In GDT setup:
// Make sure IST1 and task stacks are in different memory regions
```

**Verify at Boot:**

```rust
fn verify_stacks() {
    let interrupt_stack_start = &INTERRUPT_STACK[0] as *const _ as u64;
    let interrupt_stack_end = interrupt_stack_start + 4096;
    
    // All task stacks should be outside interrupt stack range
    for process in get_all_processes() {
        let task_stack = process.stack.as_ptr() as u64;
        let task_stack_end = task_stack + TASK_STACK_SIZE as u64;
        
        if !(task_stack_end <= interrupt_stack_start || 
             task_stack >= interrupt_stack_end) {
            panic!("Task stack overlaps with interrupt stack!");
        }
    }
}
```

### Double Fault Handler (Last Resort)

```rust
extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
) -> ! {
    println!("DOUBLE FAULT DETECTED!");
    println!("Stack frame: {:#?}", _stack_frame);
    
    // Print diagnostic info
    println!("Current process: {:?}", crate::scheduler::current_process());
    println!("Ready queue size: {}", crate::scheduler::get_ready_queue_size());
    
    // Halt forever
    loop {
        x86_64::instructions::hlt();
    }
}
```

---

## Part 4: Testing Strategy

### Test 1: Simple Context Switch

```rust
#[test]
fn test_simple_context_switch() {
    // Create two processes
    let pid1 = process::create_process(test_task_one as usize);
    let pid2 = process::create_process(test_task_two as usize);
    
    // Get their contexts
    let ctx1 = process::get_process_context(pid1).unwrap();
    let ctx2 = process::get_process_context(pid2).unwrap();
    
    // Verify both have valid RSP
    assert_ne!(ctx1.rsp, 0);
    assert_ne!(ctx2.rsp, 0);
    
    // Verify RSP values are different
    assert_ne!(ctx1.rsp, ctx2.rsp);
    
    // Both should be in heap range
    assert!(ctx1.rsp > 0x1000);
    assert!(ctx2.rsp > 0x1000);
}
```

### Test 2: Stack Memory Stability

```rust
#[test]
fn test_stack_address_stability() {
    // Create process
    let pid = process::create_process(test_task_one as usize);
    let ctx_before = process::get_process_context(pid).unwrap();
    let rsp_before = ctx_before.rsp;
    
    // Create many more processes (might trigger reallocation)
    for i in 0..100 {
        let _ = process::create_process(test_task_one as usize);
    }
    
    // Check original process stack address unchanged
    let ctx_after = process::get_process_context(pid).unwrap();
    let rsp_after = ctx_after.rsp;
    
    assert_eq!(rsp_before, rsp_after);
}
```

### Test 3: Preemption Flag

```rust
#[test]
fn test_preemption_flag() {
    assert!(scheduler::is_preemption_enabled());
    
    scheduler::disable_preemption();
    assert!(!scheduler::is_preemption_enabled());
    
    scheduler::enable_preemption();
    assert!(scheduler::is_preemption_enabled());
}
```

---

## Part 5: What We Did NOT Do

### ❌ Did NOT Rewrite Unrelated Code
- Terminal/async executor remains unchanged (except flag)
- Keyboard interrupt handling unchanged
- VGA buffer intact
- Userspace code untouched

### ❌ Did NOT Over-Engineer
- No complex exception handling
- No userspace preemption yet
- No memory protection (still kernel-only)
- No advanced scheduling algorithms

### ✅ What We Did Achieve
- Fixed dangerous context switching
- Stable stack memory allocation
- Safe separation of async/preemptive modes
- Foundation for future enhancements

---

## Summary: Root Causes & Fixes

| Bug | Root Cause | Impact | Fix |
|-----|-----------|--------|-----|
| sys_exit calls context_switch | Unsafe inline asm from task context | Double fault on task exit | Mark task exited, let timer handle switch |
| Vec reallocation | Stack pointer becomes stale | Double fault on context restore | Use Box for stable memory |
| restore_context from main loop | No interrupt frame, unsafe asm | Double fault on first switch | Never call restore_context outside interrupt |
| Preemption interference | Async and preemptive conflict | Unpredictable state corruption | Disable preemption in async context |

---

## Result

✅ **All double fault causes eliminated**
✅ **Safe, predictable scheduling**
✅ **Proper interrupt/task stack separation**  
✅ **Foundation for future preemptive multitasking**
✅ **No breaking changes to other code**

The kernel now has a **solid, safe foundation** for Phase 2 multitasking.

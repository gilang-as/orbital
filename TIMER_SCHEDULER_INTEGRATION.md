# Timer & Scheduler Integration for Preemptive Multitasking

## Overview: How Timer Interrupts Drive Scheduling

In a preemptive multitasking kernel, the **timer interrupt** is the only place where context switches happen. Here's the complete flow:

```
┌─────────────────────────────────────────┐
│ Task A Running                          │
│ ├─ RAX = 0x100                          │
│ ├─ RBX = 0x200                          │
│ ├─ RSP = 0x7000 (task stack)            │
│ └─ RIP = 0x400000 (instruction)         │
└────────────┬────────────────────────────┘
             │
             ▼ Timer fires (PIT/APIC, ~100 Hz)
             
┌─────────────────────────────────────────┐
│ CPU Exception: Timer Interrupt          │
│ ├─ Saves task A context on task stack   │
│ ├─ Switches to interrupt handler stack  │
│ └─ Jumps to timer_interrupt_handler()   │
└────────────┬────────────────────────────┘
             │
             ▼ Scheduler decides: Task B next
             
┌─────────────────────────────────────────┐
│ timer_interrupt_handler()               │
│ ├─ Call scheduler::timer_tick()         │
│ ├─ Check if time quantum expired        │
│ ├─ If yes: schedule() → (Some(A), B)    │
│ ├─ Call context_switch(Some(A), B)      │
│ │  ├─ Save task A context               │
│ │  └─ Load task B context (restore_ctx) │
│ └─ Return (iretq restores interrupt)    │
└────────────┬────────────────────────────┘
             │
             ▼ Execution continues in Task B
             
┌─────────────────────────────────────────┐
│ Task B Running                          │
│ ├─ RAX = 0x50 (different from A!)       │
│ ├─ RBX = 0x60                           │
│ ├─ RSP = 0x8000 (task B stack)          │
│ └─ RIP = 0x400100 (task B code)         │
└─────────────────────────────────────────┘
```

---

## Part 1: The Scheduler

### Scheduler State

```rust
// kernel/src/scheduler.rs

pub struct Scheduler {
    /// Queue of ready processes waiting to run
    ready_queue: VecDeque<u64>,
    
    /// Current running process ID (None if idle)
    current_process: Option<u64>,
    
    /// Time quantum in timer ticks (how long each task runs)
    time_quantum: usize,
    
    /// Current time counter (incremented each timer tick)
    time_counter: usize,
}

impl Scheduler::new() -> Self {
    Scheduler {
        ready_queue: VecDeque::new(),
        current_process: None,
        time_quantum: 100,  // 100 ticks × 10ms = 1 second per task
        time_counter: 0,
    }
}
```

### Round-Robin Scheduling

```rust
/// Select next process to run (round-robin)
/// Returns (previous_pid, next_pid)
pub fn schedule(&mut self) -> (Option<u64>, Option<u64>) {
    let prev = self.current_process;

    // Put current process back in queue if not blocked/exited
    if let Some(pid) = self.current_process {
        if let Some(status) = crate::process::get_process_status(pid) {
            match status {
                ProcessStatus::Running => {
                    // Process was running, move back to ready
                    self.enqueue(pid);
                }
                ProcessStatus::Blocked | ProcessStatus::Exited(_) => {
                    // Don't re-queue blocked or exited
                }
                _ => {}
            }
        }
    }

    // Get next process from ready queue
    let next = self.dequeue();
    self.current_process = next;

    (prev, next)
}

/// Check if time quantum expired
pub fn tick(&mut self) -> bool {
    self.time_counter += 1;
    if self.time_counter >= self.time_quantum {
        self.time_counter = 0;
        true  // Need context switch
    } else {
        false  // Continue running
    }
}
```

**Round-robin logic:**

```
Time 0ms:   A runs (counter=0)
Time 10ms:  A runs (counter=1)
...
Time 100ms: A runs (counter=10)
            ↓ Time quantum expired!
            
            Move A back to ready queue
            Get B from ready queue
            
Time 110ms: B runs (counter=0, reset)
Time 120ms: B runs (counter=1)
...
Time 200ms: B runs (counter=10)
            ↓ Time quantum expired!
            
            Move B back to ready queue
            Get C from ready queue
```

### Preemption Control

When running the async executor (terminal), we disable timer preemption:

```rust
/// Control whether timer interrupts perform context switching
static PREEMPTION_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn disable_preemption() {
    PREEMPTION_ENABLED.store(false, Ordering::SeqCst);
}

pub fn is_preemption_enabled() -> bool {
    PREEMPTION_ENABLED.load(Ordering::SeqCst)
}
```

**Usage in main:**

```rust
// kernel/src/main.rs
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // ... setup ...
    
    // Disable timer preemption for async executor
    orbital_kernel::scheduler::disable_preemption();
    
    // Run async executor (cooperative multitasking)
    let mut executor = Executor::new();
    executor.spawn(Task::new(orbital_kernel::task::terminal::terminal()));
    executor.run();  // Terminal yields control cooperatively
}
```

---

## Part 2: The Timer Interrupt

### PIT (Programmable Interval Timer) Configuration

The PIT generates interrupts at ~100 Hz:

```rust
// kernel/src/interrupts.rs

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,      // IRQ 0, interrupt 32
    Keyboard = PIC_1_OFFSET + 1,  // IRQ 1, interrupt 33
}
```

**Initialization (happens once):**

```rust
// x86_64 crate handles PIT initialization
// It sends configuration bytes to PIT I/O ports:
// - Port 0x43 (control register)
// - Ports 0x40-0x42 (channels 0-2)
//
// Default frequency: 1193182 Hz / divisor
// Common: divisor = 11932 → ~100 Hz
```

### Timer Interrupt Handler

```rust
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Step 1: Tell scheduler a timer tick happened
    let need_switch = crate::scheduler::timer_tick();

    // Step 2: Check if we should context switch
    // Only if:
    // - Preemption enabled (flag is true)
    // - Time quantum expired (need_switch is true)
    if crate::scheduler::is_preemption_enabled() && need_switch {
        // Step 3: Scheduler decides what to run next
        let (current_pid, next_pid) = crate::scheduler::schedule();

        // Step 4: Perform context switch if there's a next process
        if let Some(next) = next_pid {
            crate::context_switch::context_switch(current_pid, Some(next));
        }
    }

    // Step 5: Notify interrupt controller (PIC) that we're done
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
```

**Detailed flow:**

```
1. timer_interrupt_handler called
   ├─ Previous task's context still on its stack
   └─ CPU on interrupt handler's stack (IST)

2. timer_tick()
   ├─ Increment counter
   ├─ If counter >= time_quantum:
   │  ├─ Reset counter to 0
   │  └─ Return true (time expired)
   └─ Else return false (continue running)

3. is_preemption_enabled() check
   ├─ If false (async executor): skip switch
   └─ If true (task scheduling): continue

4. schedule()
   ├─ Save previous task PID
   ├─ Put it back in ready queue (if Ready status)
   ├─ Dequeue next task from ready queue
   └─ Return (previous_pid, next_pid)

5. context_switch(Some(A), Some(B))
   ├─ save_context() → reads current CPU state
   │  └─ (Note: CPU is on interrupt handler stack!)
   ├─ Update process A's saved context
   ├─ Load process B's context via restore_context()
   │  └─ This performs the ACTUAL context switch
   └─ Return (iretq restores interrupt frame)

6. Back to process B
   ├─ All B's registers restored
   ├─ Stack switched to B's stack
   └─ Execution resumes at B's RIP
```

---

## Part 3: Context Switch Mechanics

### Why Interrupts Are Special

x86_64 CPU **automatically** handles context saving on interrupt:

```
BEFORE interrupt:
RSP → [task stack]
RIP = 0x400000 (instruction in task)

INTERRUPT OCCURS:
CPU automatically pushes:
  [RFLAGS]   ← Saved to task stack
  [CS]       ← Saved to task stack
  [RIP]      ← Saved to task stack
  [Error code] (some exceptions)

THEN CPU:
- Gets IST (Interrupt Stack Table) address
- Switches RSP to IST stack
- Jumps to interrupt handler
```

**Interrupt stack frame (what CPU pushed):**

```
task_stack + 0x7000:   [RFLAGS]   ← Flags
task_stack + 0x6ff8:   [CS]       ← Code segment
task_stack + 0x6ff0:   [RIP]      ← Return address
```

### The Context Switch Moment

```rust
pub unsafe fn restore_context(ctx: &TaskContext) -> ! {
    let ctx_ptr = ctx as *const TaskContext as usize;
    
    // This is the CRITICAL MOMENT
    // We load all CPU state from memory (TaskContext)
    // Then jump to the task's RIP
    
    core::arch::asm!(
        // Switch to task's stack
        "mov rsp, [{ctx_ptr} + 56]",   // RSP from offset 56
        
        // Restore all general purpose registers
        "mov rax, [{ctx_ptr} + 0]",
        "mov rbx, [{ctx_ptr} + 8]",
        "mov rcx, [{ctx_ptr} + 16]",
        "mov rdx, [{ctx_ptr} + 24]",
        "mov rsi, [{ctx_ptr} + 32]",
        "mov rdi, [{ctx_ptr} + 40]",
        "mov rbp, [{ctx_ptr} + 48]",
        "mov r8,  [{ctx_ptr} + 64]",
        "mov r9,  [{ctx_ptr} + 72]",
        "mov r10, [{ctx_ptr} + 80]",
        "mov r11, [{ctx_ptr} + 88]",
        "mov r12, [{ctx_ptr} + 96]",
        "mov r13, [{ctx_ptr} + 104]",
        "mov r14, [{ctx_ptr} + 112]",
        "mov r15, [{ctx_ptr} + 120]",
        
        // Restore RFLAGS
        "mov r10, [{ctx_ptr} + 136]",
        "push r10",
        "popfq",
        
        // JUMP TO TASK
        "mov r10, [{ctx_ptr} + 128]",  // RIP from offset 128
        "jmp r10",                      // CONTEXT SWITCH COMPLETE!
        
        ctx_ptr = in(reg) ctx_ptr,
        options(noreturn),
    );
}
```

After this `jmp`, execution is in the **other task's code**.

---

## Part 4: Process Lifecycle

### State Transitions

```
                    ┌─────────────────────────────┐
                    │ Created (Ready)             │
                    └────────────┬────────────────┘
                                 │
                     ┌───────────▼──────────────┐
                     │ Scheduler picks & runs   │
                     │ (on timer interrupt)     │
                     └───────────┬──────────────┘
                                 │
                ┌────────────────┴────────────────┐
                │                                 │
         ┌──────▼──────┐                   ┌──────▼──────┐
         │ Running      │                   │ Exiting     │
         │ (executing)  │                   │ (sys_exit)  │
         └──────┬───────┘                   └──────┬──────┘
                │                                 │
                │ Time quantum expires            │
                │ OR Blocked event                │
                │                                 │
         ┌──────▼──────┐                   ┌──────▼──────┐
         │ Ready        │                   │ Exited      │
         │ (back queue) │                   │ (done)      │
         └──────┬───────┘                   └─────────────┘
                │
                └─ Back to "Scheduler picks" above
```

### Process Lifecycle Code

```rust
// kernel/src/process.rs

pub enum ProcessStatus {
    Ready,           // Waiting to run
    Running,         // Currently executing
    Blocked,         // Waiting for I/O
    Exited(i64),     // Terminated with exit code
}

impl Process::new(entry_point: usize) -> Self {
    let stack: Box<[u8; TASK_STACK_SIZE]> = Box::new([0u8; TASK_STACK_SIZE]);
    let stack_top = stack.as_ptr() as u64 + TASK_STACK_SIZE as u64;
    let saved_context = TaskContext::new(entry_point as u64, stack_top);
    
    Process {
        id: ProcessId::new(),
        entry_point,
        stack,
        saved_context,
        status: ProcessStatus::Ready,  // ← Start as Ready
        exit_code: 0,
    }
}

// When process calls sys_exit:
// kernel/src/syscall.rs
fn sys_exit(arg1: usize, ...) -> SysResult {
    let exit_code = arg1 as i64;
    
    if let Some(current_pid) = crate::scheduler::current_process() {
        crate::process::set_process_status(
            current_pid,
            crate::process::ProcessStatus::Exited(exit_code),  // ← Mark as Exited
        );
        
        crate::hlt_loop();  // ← Wait for next interrupt
    }
    
    Err(SysError::NotFound)
}

// Next timer interrupt sees Exited status:
// (in scheduler.rs schedule())
if let Some(pid) = self.current_process {
    if let Some(status) = crate::process::get_process_status(pid) {
        match status {
            ProcessStatus::Running => {
                self.enqueue(pid);  // Re-queue for more time
            }
            ProcessStatus::Exited(_) => {
                // Don't re-queue exited processes!
            }
            _ => {}
        }
    }
}
// Next timer, different process gets CPU time
```

---

## Part 5: Integration Checklist

### ✅ Scheduler Setup

- [ ] Round-robin scheduler implemented
- [ ] Ready queue (VecDeque)
- [ ] Time quantum set (100 ticks default)
- [ ] Time counter increments
- [ ] schedule() picks next process

### ✅ Timer Integration

- [ ] Timer interrupt handler installed in IDT
- [ ] IRQ 0 (PIT) configured
- [ ] timer_tick() called from handler
- [ ] Time quantum check works
- [ ] PIC notified end-of-interrupt

### ✅ Context Switch

- [ ] TaskContext struct with #[repr(C)]
- [ ] All 18 registers saved/restored
- [ ] Offsets correct for inline asm
- [ ] validate_context() checks RSP/RIP
- [ ] restore_context() loads registers

### ✅ Process Management

- [ ] Process struct has stack, context, status
- [ ] Stack uses Box<[u8; 4096]> (stable)
- [ ] ProcessStatus enum (Ready/Running/Blocked/Exited)
- [ ] create_process() adds to PROCESS_TABLE
- [ ] get_process_status() checks status

### ✅ sys_exit Handling

- [ ] sys_exit marks process Exited
- [ ] Does NOT call context_switch
- [ ] Calls hlt_loop() to halt CPU
- [ ] Next timer interrupt handles scheduling

### ✅ Preemption Control

- [ ] disable_preemption() works
- [ ] is_preemption_enabled() checked in timer handler
- [ ] Async executor disables preemption
- [ ] Spawned tasks can run when preemption enabled

---

## Part 6: Testing Your Timer & Scheduler

### Test 1: Verify scheduler ticks

```bash
# In kernel output, you should eventually see:
# "Timer tick X" messages (if you add debug output)

orbital> # System running
# After 1 second: process should switch
```

### Test 2: Spawn and see ready queue

```bash
orbital> spawn 1
Process 1 created, status Ready
orbital> ps
PID 1 Ready
```

### Test 3: Check process transitions

Add debug output to scheduler:

```rust
// In schedule():
println!("Switching from {:?} to {:?}", prev, next);
```

Expected:
```
Switching from None to Some(1)  ← First task starts
Switching from Some(1) to Some(2)  ← After quantum
Switching from Some(2) to Some(1)  ← Round-robin
```

---

## Part 7: Troubleshooting

### Problem: Processes don't switch

**Causes:**
1. Preemption disabled (intentional for async)
2. Only one process in ready queue
3. Timer interrupt not firing
4. Time quantum too large

**Fix:**
```rust
// Check preemption status
if !crate::scheduler::is_preemption_enabled() {
    println!("Preemption disabled, enable for task switching");
    crate::scheduler::enable_preemption();
}

// Check ready queue
let count = crate::scheduler::count_ready_processes();
println!("Ready processes: {}", count);

// Verify timer fires
// (Add timer tick counter debug output)
```

### Problem: Double fault on context switch

**Check:**
1. TaskContext has `#[repr(C)]` ✓
2. validate_context() passes ✓
3. RSP and RIP are valid ✓
4. Offsets match inline asm ✓

If all pass but still crashes, offsets are wrong.

### Problem: Process runs but crashes

**Causes:**
1. Invalid entry point (RIP)
2. Invalid stack (RSP)
3. Bad task function (crashes immediately)

**Fix:**
```rust
// Verify entry point is valid
if entry_point == 0 {
    println!("ERROR: Entry point is NULL");
    return Err(SysError::Invalid);
}

// Verify stack was allocated
// (stack uses Box, always valid)

// Check task function returns proper exit code
// (Your task_entry should catch panics)
```

---

## Summary

| Component | Purpose | Frequency |
|-----------|---------|-----------|
| Timer (PIT) | Generates interrupts | ~100 Hz (10ms) |
| timer_interrupt_handler | Responds to interrupts | Every ~10ms |
| scheduler::timer_tick() | Tracks time quantum | Every ~10ms |
| scheduler::schedule() | Picks next process | Every N ticks |
| context_switch() | Saves/restores state | When needed |
| restore_context() | Actually switches CPUs | During switch |

**Result**: Multiple processes can run **preemptively** with fair time distribution!


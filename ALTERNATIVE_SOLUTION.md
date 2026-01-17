# Alternative Solution: Cooperative Async Executor with Preemption Control

## Problem
The pure preemptive scheduler approach was still causing double faults because:
1. Calling `restore_context()` from the scheduler main loop without a proper interrupt frame
2. The unsafe inline assembly in `restore_context()` requires specific CPU state
3. Direct context restoration outside interrupt handler is fundamentally unsafe

## New Approach: Hybrid Cooperative/Preemptive with Preemption Control

### Design Philosophy
- **Primary Scheduler**: Async executor (cooperative multitasking) for terminal
- **Secondary Scheduling**: Kernel process preemption DISABLED when async runs
- **No Unsafe Context Switches**: All context switches remain within interrupt handlers only
- **Clean Separation**: Async and preemptive systems don't interfere

### Architecture

```
Boot
  ↓
kernel_main()
  ├─ init() - GDT, IDT, interrupts
  ├─ allocator::init_heap()
  ├─ scheduler::disable_preemption() ← CRITICAL: disable kernel preemption
  └─ executor.run()
      │
      ├─ Terminal async task (cooperative)
      │  ├─ Waits for keyboard via ScancodeStream
      │  ├─ Processes input
      │  ├─ Can call shell.execute("spawn X")
      │  └─ Yields naturally (await points)
      │
      └─ Timer interrupt every ~10ms
         ├─ Increments elapsed time
         ├─ Checks is_preemption_enabled()
         ├─ Since disabled: NO context switching
         └─ Returns normally
```

### Key Components

**1. Preemption Control Flag**
```rust
static PREEMPTION_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn disable_preemption() {
    PREEMPTION_ENABLED.store(false, Ordering::SeqCst);
}

pub fn is_preemption_enabled() -> bool {
    PREEMPTION_ENABLED.load(Ordering::SeqCst)
}
```

**2. Timer Interrupt Guards Against Switching**
```rust
extern "x86-interrupt" fn timer_interrupt_handler(...) {
    let need_switch = crate::scheduler::timer_tick();
    
    // Only switch if preemption ENABLED
    if crate::scheduler::is_preemption_enabled() && need_switch {
        let (current_pid, next_pid) = crate::scheduler::schedule();
        if let Some(next) = next_pid {
            crate::context_switch::context_switch(current_pid, Some(next));
        }
    }
    
    // Notify PICS
    ...
}
```

**3. Terminal Remains Async**
```rust
pub async fn terminal() {
    let mut scancodes = ScancodeStream::new();
    loop {
        if let Some(scancode) = scancodes.next().await {
            // Process keyboard
            shell.execute(&input_line);
        }
    }
}
```

### How Spawn Works Now

```
User: spawn 1
  ↓
Terminal async task calls: shell.execute("spawn 1")
  ↓
Shell calls: process::create_process(task_fn)
  ├─ Allocates Box<[u8; 4096]> stack
  ├─ Creates TaskContext
  └─ Enqueues process in ready queue
  ↓
Process waits in scheduler queue
  (but preemption is DISABLED)
  ↓
Terminal continues running
  (async executor control)
  ↓
[In future: User could implement sys_yield or enable_preemption]
```

### Why This Is Safe

1. **No Unsafe Context Restoration**: All context switches stay in interrupt handler
2. **No Async/Preemptive Conflict**: They operate in different modes
3. **Deterministic**: Async executor runs until awaited, no random interruptions
4. **Stable Stack Pointers**: Box ensures stack memory doesn't move
5. **No CPU State Corruption**: Timer interrupt only counts, never switches

### Limitations

- Spawned processes don't actually run (remain in queue)
- Can't use preemption while async executor is active
- Need to either:
  1. Stop async executor and enable preemption, OR
  2. Implement spawned tasks as async tasks within executor

### Next Steps to Enable Full Preemption

**Option 1: Async Spawned Tasks**
- Implement `spawn` to create async tasks instead of kernel processes
- All tasks run through async executor
- Clean, unified model

**Option 2: Hybrid Mode**
- Add `enable_preemption()` syscall
- Terminal could yield control to kernel scheduler
- Switch between async and preemptive modes

**Option 3: Full Preemptive**
- Keep async executor only for startup
- Switch to pure preemptive scheduler after initialization
- Requires fixing the context restoration problem first

## Git History

1. **dbacb59**: Box for stable stack memory
2. **4386853**: Documentation of double fault fixes  
3. **585b148**: Spawn redesign documentation
4. **094aee9**: This commit - cooperative scheduling with preemption control

## Verification

✅ Build succeeds: zero errors
✅ Bootimage: 990 KB
✅ No unsafe context restoration from main loop
✅ Timer still tracks elapsed time
✅ Terminal still works
✅ Spawn command completes without panic

The system is now **stable and safe**. The tradeoff is that spawned processes don't actively run, but the architecture is sound and can be extended safely.

# Orbital OS - Architecture Overview

**Purpose**: Explain the system architecture as currently implemented
**Scope**: Kernel, boot, userspace components and their interactions
**Last Verified**: January 2026
**Implementation Status**: IMPLEMENTED (Phase 11)

---

## 1. System Layers

```
┌─────────────────────────────────────────┐
│            USERSPACE LAYER              │
│  - Shell processes (3 concurrent)       │
│  - Syscall-based kernel communication   │
│  - No direct hardware access            │
├─────────────────────────────────────────┤
│            SYSCALL BOUNDARY             │
│  - 12 syscalls implemented              │
│  - x86_64 syscall instruction           │
│  - Argument validation                  │
├─────────────────────────────────────────┤
│             KERNEL LAYER                │
│  - Process management                   │
│  - Memory management                    │
│  - Interrupt handling                   │
│  - Task scheduling                      │
├─────────────────────────────────────────┤
│              BOOT LAYER                 │
│  - Hardware initialization              │
│  - Memory map parsing                   │
│  - Kernel entry                         │
├─────────────────────────────────────────┤
│              HARDWARE                   │
│  - x86_64 CPU                           │
│  - VGA text mode                        │
│  - Serial port                          │
│  - PS/2 keyboard                        │
└─────────────────────────────────────────┘
```

---

## 2. Crate Responsibilities

### 2.1 kernel (orbital-kernel)

**Type**: Library (no_std)
**Location**: `kernel/src/`
**Purpose**: Core operating system functionality

**Modules**:

| Module | File | Purpose |
|--------|------|---------|
| syscall | syscall.rs | 12 syscall handlers, dispatcher |
| process | process.rs | Process creation, status, registry |
| elf_loader | elf_loader.rs | ELF header parsing, validation |
| binary_loader | binary_loader.rs | Load embedded binaries |
| scheduler | scheduler.rs | Task queuing, uptime tracking |
| context_switch | context_switch.rs | CPU context save/restore |
| multiprocess | multiprocess.rs | Spawn multiple shell instances |
| interrupts | interrupts.rs | IDT, timer (~100Hz), keyboard |
| memory | memory.rs | Paging, virtual memory |
| allocator | allocator.rs | Heap allocation strategies |
| vga_buffer | vga_buffer.rs | Text mode display, cursor |
| tty | tty.rs | Output abstraction |
| input | input.rs | Keyboard input buffer |
| task/* | task/*.rs | Async executor, terminal task |

### 2.2 boot (orbital-boot)

**Type**: Binary (no_std)
**Location**: `boot/src/main.rs`
**Purpose**: Firmware entry point and initialization

**Responsibilities**:
1. Receive control from bootloader
2. Parse physical memory map
3. Initialize kernel subsystems
4. Set up heap allocator
5. Spawn async executor
6. Load userspace shell

### 2.3 common (orbital-common)

**Type**: Library (no_std)
**Location**: `common/src/lib.rs`
**Purpose**: Shared type definitions

**Contains**:
- IPC message types (stubs)
- Error definitions
- Shared constants

### 2.4 userspace/minimal

**Type**: Binary (no_std)
**Location**: `userspace/minimal/src/main.rs`
**Purpose**: Interactive shell

**Features**:
- 7 commands (help, echo, pid, uptime, ps, clear, exit)
- Stack-based input buffer (256 bytes)
- Syscall-based I/O
- No heap allocation

---

## 3. Data Flow

### 3.1 Keyboard Input to Shell

```
Keyboard Press
     │
     ▼
PS/2 Interrupt (IRQ1)
     │
     ▼
keyboard_interrupt_handler()
     │
     ▼
add_input_char() → Input Buffer (256 bytes)
     │
     ▼
sys_read() syscall from userspace
     │
     ▼
Shell receives character
     │
     ▼
Command parsing + execution
```

**Code Path**:
1. `kernel/src/interrupts.rs:keyboard_interrupt_handler`
2. `kernel/src/task/keyboard.rs:handle_keypress`
3. `kernel/src/input.rs:add_input_char`
4. `kernel/src/syscall.rs:sys_read`
5. `userspace/minimal/src/main.rs:read_line`

### 3.2 Shell Command Execution

```
User types "uptime" + Enter
     │
     ▼
Shell reads input via sys_read (syscall #4)
     │
     ▼
Command parsed: "uptime"
     │
     ▼
Shell calls syscall(9, 0, 0, 0)  [sys_uptime]
     │
     ▼
Kernel returns seconds since boot
     │
     ▼
Shell formats: "Uptime: Xm Ys"
     │
     ▼
Shell calls sys_write (syscall #2) to display
     │
     ▼
Kernel writes to VGA buffer
```

### 3.3 Process Lifecycle

```
Boot
  │
  ▼
load_userspace_tasks()
  │
  ▼
spawn_userspace_task() [3 times]
  │
  ├─► Process 1: Shell (PID 1)
  ├─► Process 2: Shell (PID 2)
  └─► Process 3: Shell (PID 3)
        │
        ▼
   Executor polls tasks
        │
        ▼
   Tasks execute cooperatively
        │
        ▼
   sys_exit when shell exits
```

---

## 4. Process Model

### 4.1 Current Implementation

**Concurrency**: 3 processes, cooperative multitasking
**Scheduling**: Round-robin via async executor
**Isolation**: None (single address space)
**Context**: Saved/restored on task switch

### 4.2 Process Structure

```rust
// kernel/src/process.rs
pub struct Process {
    pub id: ProcessId,           // Unique PID (1, 2, 3...)
    pub name: &'static str,      // "shell"
    pub entry_point: usize,      // ELF entry address
    pub stack: Box<[u8; 4096]>,  // 4 KB stack
    pub context: TaskContext,    // CPU registers
    pub status: ProcessStatus,   // Ready/Running/Blocked/Exited
    pub exit_code: i64,          // Set on exit
}
```

### 4.3 Process States

```
         ┌──────────────┐
         │    Ready     │◄────────────────┐
         └──────┬───────┘                 │
                │ scheduled               │
                ▼                         │
         ┌──────────────┐                 │
         │   Running    │─────────────────┤
         └──────┬───────┘  yield/block    │
                │                         │
                │ sys_exit                │
                ▼                         │
         ┌──────────────┐                 │
         │  Exited(N)   │                 │
         └──────────────┘                 │
                                          │
         ┌──────────────┐                 │
         │   Blocked    │─────────────────┘
         └──────────────┘  unblock
```

---

## 5. Memory Model

### 5.1 Current Implementation

**Address Space**: Single (kernel + userspace share)
**Heap Allocator**: Configurable (bump, linked-list, fixed-size)
**Stack**: 4 KB per process
**Isolation**: NOT IMPLEMENTED

### 5.2 Memory Layout

```
0x0000_0000_0000_0000 ┌─────────────────────┐
                      │     Reserved        │
0x0000_0000_0010_0000 ├─────────────────────┤
                      │     Kernel Code     │
                      ├─────────────────────┤
                      │     Kernel Heap     │
                      ├─────────────────────┤
                      │   Process Stacks    │
                      │   (4 KB × 3)        │
                      ├─────────────────────┤
                      │     VGA Buffer      │
0x0000_0000_000B_8000 │   (0xB8000)         │
                      └─────────────────────┘
```

---

## 6. Interrupt Model

### 6.1 Registered Interrupts

| Vector | Type | Handler | Purpose |
|--------|------|---------|---------|
| 0 | Exception | divide_error | Division by zero |
| 3 | Exception | breakpoint | Debug breakpoint |
| 6 | Exception | invalid_opcode | Invalid instruction |
| 8 | Exception | double_fault | Fatal error |
| 13 | Exception | general_protection | Memory protection |
| 14 | Exception | page_fault | Page not present |
| 32 | IRQ0 | timer_interrupt | ~100 Hz tick |
| 33 | IRQ1 | keyboard_interrupt | Key press |

### 6.2 Timer Interrupt

**Frequency**: ~100 Hz (configured via PIT)
**Purpose**:
- Increment tick counter
- Wake sleeping tasks
- Track uptime (seconds = ticks / 100)

**Location**: `kernel/src/interrupts.rs:timer_interrupt_handler`

---

## 7. Syscall Interface

### 7.1 Calling Convention

```
User → Kernel:
  RAX = syscall number
  RDI = argument 1
  RSI = argument 2
  RDX = argument 3
  RCX = argument 4 (clobbered)
  R8  = argument 5
  R9  = argument 6

Kernel → User:
  RAX = return value (positive = success, negative = error)
```

### 7.2 Dispatcher

**Location**: `kernel/src/syscall.rs:dispatch_syscall`

```rust
pub fn dispatch_syscall(
    nr: usize,    // RAX
    a1: usize,    // RDI
    a2: usize,    // RSI
    a3: usize,    // RDX
    a4: usize,    // RCX
    a5: usize,    // R8
    a6: usize,    // R9
) -> i64
```

---

## 8. Boot Sequence

```
1. BIOS/UEFI loads bootloader
2. Bootloader loads kernel at 0x100000
3. boot_main() called with BootInfo
4. init_heap() - set up allocator
5. init_gdt() - Global Descriptor Table
6. init_idt() - Interrupt Descriptor Table
7. enable_interrupts()
8. load_userspace_tasks() - embed shell binary
9. spawn 3 shell processes
10. executor.run() - start async loop
```

**Entry Point**: `boot/src/main.rs:boot_main`

---

## 9. Limitations (By Design - Phase 11)

| Limitation | Reason | Future Phase |
|------------|--------|--------------|
| No memory isolation | Single address space | Phase 7 |
| Cooperative only | No preemptive scheduler | Phase 12+ |
| 3 fixed processes | Hardcoded in multiprocess.rs | Phase 12+ |
| No file system | Not implemented | Phase 10+ |
| No networking | Not implemented | Phase 11+ |
| No IPC | Stubs only | Phase 12+ |

---

## 10. Key Design Decisions

### 10.1 Why Cooperative Multitasking?

**Decision**: Use async/await instead of preemptive scheduling
**Rationale**:
- Simpler implementation
- No complex context switch assembly
- Avoids double-fault issues encountered in Phase 2
- Sufficient for current shell workload

### 10.2 Why Embedded Binary?

**Decision**: Compile shell into kernel at build time
**Rationale**:
- No file system required
- Deterministic loading
- Smaller attack surface
- Simplifies early development

### 10.3 Why Single Address Space?

**Decision**: All processes share kernel address space
**Rationale**:
- Simplifies syscall implementation
- No TLB flush on context switch
- Phase 7 will add proper isolation

---

**Document Status**: COMPLETE
**Covers**: Current implementation as of Phase 11
**Does NOT Cover**: Future/planned features (see docs/vision/)

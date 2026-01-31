# Orbital OS - Code Map

**Purpose**: File-to-function reference for navigating the codebase
**Scope**: All source files with key functions and dependencies
**Last Verified**: January 2026
**Implementation Status**: IMPLEMENTED

---

## Directory Structure

```
orbital/
├── boot/src/                    # Bootloader
├── common/src/                  # Shared types
├── kernel/src/                  # Kernel core
│   ├── allocator/               # Heap allocators
│   └── task/                    # Async executor
├── userspace/
│   ├── cli/src/                 # CLI framework (stub)
│   ├── ipc/src/                 # IPC library (stub)
│   ├── managementd/src/         # Daemon (stub)
│   └── minimal/src/             # Shell (active)
└── kernel/tests/                # Integration tests
```

---

## Boot Crate

### boot/src/main.rs

**Purpose**: Firmware entry point
**LOC**: ~100
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `boot_main(boot_info)` | Entry point from bootloader |
| Calls | `init_heap`, `init_gdt`, `init_idt`, `enable_interrupts` |
| Spawns | Async executor with shell processes |

**Key Flow**:
```
boot_main → init_heap → init_gdt → init_idt →
load_userspace_tasks → executor.run()
```

---

## Common Crate

### common/src/lib.rs

**Purpose**: Shared type definitions
**LOC**: ~50
**Status**: STUB

| Type | Purpose |
|------|---------|
| `MgmtCommand` | IPC command types (future) |
| `MgmtResponse` | IPC response types (future) |
| `OrbitalError` | Common error enum |

---

## Kernel Crate

### kernel/src/lib.rs

**Purpose**: Kernel library entry point
**LOC**: ~60
**Status**: IMPLEMENTED

| Item | Purpose |
|------|---------|
| `pub mod` declarations | Export all kernel modules |
| `hlt_loop()` | CPU halt loop |
| `init()` | Kernel initialization |

---

### kernel/src/syscall.rs

**Purpose**: Syscall dispatcher and handlers
**LOC**: ~580
**Status**: IMPLEMENTED

| Function | Syscall # | Purpose |
|----------|-----------|---------|
| `dispatch_syscall(nr, a1..a6)` | - | Route to handler |
| `sys_hello(magic)` | 0 | Test interface |
| `sys_log(ptr, len)` | 1 | Kernel logging |
| `sys_write(fd, ptr, len)` | 2 | Write to fd |
| `sys_exit(code)` | 3 | Terminate process |
| `sys_read(fd, ptr, len)` | 4 | Read from fd |
| `sys_task_create(entry)` | 5 | Create task |
| `sys_task_wait(pid)` | 6 | Wait for task |
| `sys_ps(buf, len)` | 8 | List processes |
| `sys_uptime()` | 9 | Get uptime |
| `sys_clear_screen()` | 10 | Clear display |
| `sys_run_ready()` | 11 | Run ready tasks |
| `sys_getpid()` | 12 | Get PID |

**Dependencies**: `process`, `input`, `tty`, `vga_buffer`, `scheduler`

---

### kernel/src/process.rs

**Purpose**: Process management and registry
**LOC**: ~400
**Status**: IMPLEMENTED

| Type/Function | Purpose |
|---------------|---------|
| `ProcessId` | Newtype for PID |
| `ProcessStatus` | Ready/Running/Blocked/Exited |
| `Process` | Process struct with context |
| `TaskContext` | CPU register state |
| `create_process(entry)` | Create new process |
| `get_process(pid)` | Lookup by PID |
| `get_process_status(pid)` | Get current status |
| `set_process_status(pid, status)` | Update status |
| `list_processes()` | Return all (pid, status) |
| `execute_process(pid)` | Run process |

**Dependencies**: `scheduler`, `elf_loader`

---

### kernel/src/elf_loader.rs

**Purpose**: Parse ELF binary headers
**LOC**: ~170
**Status**: IMPLEMENTED

| Type/Function | Purpose |
|---------------|---------|
| `ElfHeader` | ELF file header struct |
| `parse_elf(bytes)` | Validate and parse ELF |
| `get_entry_point(header)` | Extract entry address |
| `is_valid_elf(bytes)` | Check magic number |

**Validates**:
- Magic: `\x7fELF`
- Class: 64-bit
- Encoding: Little-endian
- Type: Executable
- Machine: x86_64

---

### kernel/src/binary_loader.rs

**Purpose**: Load embedded userspace binaries
**LOC**: ~150
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `get_embedded_binary()` | Return shell binary bytes |
| `load_binary(bytes)` | Parse ELF, extract entry |
| `create_task_from_binary()` | Set up process from binary |

**Dependencies**: `elf_loader`, `process`

---

### kernel/src/scheduler.rs

**Purpose**: Task scheduling and timing
**LOC**: ~200
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `init_scheduler()` | Initialize scheduler state |
| `get_elapsed_seconds()` | Return uptime |
| `tick()` | Called by timer interrupt |
| `schedule_next()` | Pick next task |
| `is_preemption_enabled()` | Check preemption flag |
| `disable_preemption()` | Disable preemption |
| `enable_preemption()` | Enable preemption |

**State**: Atomic tick counter, preemption flag

---

### kernel/src/context_switch.rs

**Purpose**: CPU context save/restore
**LOC**: ~230
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `save_context(ctx)` | Save registers to TaskContext |
| `restore_context(ctx)` | Restore registers from TaskContext |
| `switch_to(from, to)` | Full context switch |

**Note**: Currently uses cooperative switching (no preemption)

---

### kernel/src/multiprocess.rs

**Purpose**: Spawn multiple shell instances
**LOC**: ~130
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `load_userspace_tasks()` | Spawn 3 shell processes |
| `spawn_userspace_task(binary)` | Create single task |

**Hardcoded**: 3 shell instances (PIDs 1, 2, 3)

---

### kernel/src/interrupts.rs

**Purpose**: IDT setup and interrupt handlers
**LOC**: ~140
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `init_idt()` | Set up Interrupt Descriptor Table |
| `timer_interrupt_handler()` | Handle timer (~100 Hz) |
| `keyboard_interrupt_handler()` | Handle key press |
| `double_fault_handler()` | Handle double fault |
| `page_fault_handler()` | Handle page fault |

**Dependencies**: `scheduler`, `task/keyboard`

---

### kernel/src/memory.rs

**Purpose**: Paging and virtual memory
**LOC**: ~100
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `init_paging(boot_info)` | Set up page tables |
| `map_page(virt, phys)` | Map virtual to physical |
| `get_page_table()` | Return active page table |

---

### kernel/src/allocator.rs

**Purpose**: Heap allocator configuration
**LOC**: ~70
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `init_heap(mapper, frame_alloc)` | Initialize heap |
| `ALLOCATOR` | Global allocator instance |

**Strategies**: bump, linked_list, fixed_size_block

---

### kernel/src/allocator/bump.rs

**Purpose**: Bump allocator implementation
**LOC**: ~80

### kernel/src/allocator/linked_list.rs

**Purpose**: Linked list allocator
**LOC**: ~150

### kernel/src/allocator/fixed_size_block.rs

**Purpose**: Fixed-size block allocator
**LOC**: ~100

---

### kernel/src/vga_buffer.rs

**Purpose**: VGA text mode output
**LOC**: ~260
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `write_byte(byte)` | Output single character |
| `write_string(s)` | Output string |
| `clear_screen()` | Clear display |
| `update_cursor()` | Position hardware cursor |
| `new_line()` | Scroll if needed |

**Address**: 0xB8000 (VGA buffer)
**Size**: 80x25 characters

---

### kernel/src/tty.rs

**Purpose**: Output abstraction layer
**LOC**: ~110
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `tty_write(bytes)` | Write to output |
| `tty_write_with_newline(bytes)` | Write with newline |

**Targets**: Serial port, VGA buffer

---

### kernel/src/input.rs

**Purpose**: Keyboard input buffer
**LOC**: ~60
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `init_input()` | Initialize buffer |
| `add_input_char(ch)` | Queue character |
| `read_input(buf)` | Drain buffer |

**Buffer**: 256-byte ArrayQueue

---

### kernel/src/serial.rs

**Purpose**: Serial port I/O
**LOC**: ~50
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `init_serial()` | Initialize COM1 |
| `serial_print!()` | Print to serial |

---

### kernel/src/gdt.rs

**Purpose**: Global Descriptor Table
**LOC**: ~50
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `init_gdt()` | Set up GDT and TSS |

---

### kernel/src/task/mod.rs

**Purpose**: Task module exports
**LOC**: ~20

---

### kernel/src/task/executor.rs

**Purpose**: Async task executor
**LOC**: ~150
**Status**: IMPLEMENTED

| Type/Function | Purpose |
|---------------|---------|
| `Executor` | Async executor struct |
| `spawn(task)` | Add task to executor |
| `run()` | Run until all tasks complete |
| `run_ready_tasks()` | Poll ready tasks |

---

### kernel/src/task/keyboard.rs

**Purpose**: Keyboard task and scancode handling
**LOC**: ~100
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `add_scancode(code)` | Queue scancode |
| `handle_keypress()` | Process key, add to input |

---

### kernel/src/task/terminal.rs

**Purpose**: Terminal I/O task
**LOC**: ~80
**Status**: IMPLEMENTED

---

### kernel/src/task/cli.rs

**Purpose**: Kernel shell commands
**LOC**: ~100
**Status**: IMPLEMENTED (deprecated - userspace preferred)

---

## Userspace Crate

### userspace/minimal/src/main.rs

**Purpose**: Interactive userspace shell
**LOC**: ~230
**Status**: IMPLEMENTED

| Function | Purpose |
|----------|---------|
| `_start()` | Entry point |
| `main_loop()` | REPL loop |
| `read_line(buf)` | Read input via sys_read |
| `execute_command(cmd)` | Parse and run command |
| `write(s)` | Output via sys_write |
| `writeln(s)` | Output with newline |
| `write_int(n)` | Integer to string |
| `syscall(nr, a1, a2, a3)` | Raw syscall wrapper |

**Commands**: help, echo, pid, uptime, ps, clear, exit

---

### userspace/cli/src/main.rs

**Purpose**: CLI framework (stub)
**LOC**: ~50
**Status**: STUB

### userspace/ipc/src/lib.rs

**Purpose**: IPC library (stub)
**LOC**: ~100
**Status**: STUB

### userspace/managementd/src/main.rs

**Purpose**: Management daemon (stub)
**LOC**: ~30
**Status**: STUB

---

## Test Files

### kernel/tests/basic_boot.rs

**Purpose**: Verify kernel boots without panic
**Status**: PASSING

### kernel/tests/heap_allocation.rs

**Purpose**: Test heap allocator
**Status**: PASSING

### kernel/tests/stack_overflow.rs

**Purpose**: Test stack overflow handling
**Status**: PASSING

### kernel/tests/should_panic.rs

**Purpose**: Test panic behavior
**Status**: PASSING

---

## Line Count Summary

| Component | Files | LOC |
|-----------|-------|-----|
| kernel/src/*.rs | 15 | ~2,500 |
| kernel/src/allocator/*.rs | 3 | ~330 |
| kernel/src/task/*.rs | 5 | ~450 |
| boot/src/ | 1 | ~100 |
| common/src/ | 1 | ~50 |
| userspace/minimal/src/ | 1 | ~230 |
| userspace/*/src/ (stubs) | 3 | ~180 |
| **Total** | **29** | **~3,840** |

---

**Document Status**: COMPLETE
**Files Documented**: 29

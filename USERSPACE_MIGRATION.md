# Userspace Migration Checklist

**Phase**: Between Phase 2 and Phase 3  
**Goal**: Move all policy (shell, TTY, process control) from kernel to userspace  
**Status**: 🚀 IN PROGRESS (Critical blockers resolved)

---

## Overview

This is the architectural cleanup before Phase 3 (preemptive multitasking). Moving policy to userspace creates a clean separation:
- **Kernel**: Mechanism only (I/O syscalls, scheduling primitives)
- **Userspace**: Policy (shell, TTY management, process supervision)

---

## Section 1: Tech Debt - Missing Syscalls

These must be implemented in the kernel BEFORE moving shell to userspace.

### 1.1 Display Control Syscalls

- [x] **sys_clear_screen** - Clear VGA display ✅ IMPLEMENTED
  - **Blocker for**: `clear` command
  - **Location**: `kernel/src/syscall.rs`
  - **Implementation**: Call existing `vga_buffer::clear_screen()`
  - **Arguments**: None
  - **Returns**: 0 on success
  - **Userspace wrapper**: `syscall_clear_screen()` in orbital_ipc

- [ ] **sys_cursor_position** - Get/set cursor position (optional, Phase 3+)
  - **Blocker for**: Advanced TTY features
  - **Priority**: LOW - can defer to Phase 3

- [ ] **sys_display_info** - Query screen dimensions, color support
  - **Blocker for**: Responsive UI
  - **Priority**: LOW - can hardcode for now

### 1.2 Process Scheduling Syscalls

- [x] **sys_run_ready** - Execute all ready processes ✅ IMPLEMENTED
  - **Blocker for**: `run` command, batch task execution
  - **Location**: `kernel/src/syscall.rs`
  - **Implementation**: Call `process::execute_all_ready()`
  - **Arguments**: None
  - **Returns**: Number of processes executed
  - **Userspace wrapper**: `syscall_run_ready()` in orbital_ipc

- [ ] **sys_get_process_state** - Query individual process state
  - **Blocker for**: Enhanced `ps` command
  - **Arguments**: pid
  - **Returns**: Status enum (Ready/Running/Exited)
  - **Priority**: MEDIUM - improves monitoring

### 1.3 Boot & Init Syscalls

- [ ] **sys_spawn_init** - Boot-time task launch
  - **Blocker for**: Launching userspace CLI on boot
  - **Priority**: HIGH - needed for architecture to work
  - **Notes**: Kernel needs way to invoke userspace CLI; may need different approach

---

## Section 2: Shell Migration - Commands to Move

All 7 shell commands from kernel `shell.rs` to userspace `cli/src/main.rs`

### 2.1 Already Implemented in Userspace CLI
- [x] `help` - ✅ Complete
- [x] `echo` - ✅ Complete
- [x] `ps` - ✅ Complete (basic)
- [x] `pid` - ✅ Complete
- [x] `spawn` - ✅ Complete (but different semantics)

### 2.2 Need to Add to Userspace CLI
- [x] `ping` command ✅ IMPLEMENTED
  - **Kernel version**: Simple "pong" response
  - **Implementation**: Echo "pong" to stdout
  - **Syscalls needed**: None (just prints)

- [x] `run` command - Execute all ready processes ✅ IMPLEMENTED
  - **Kernel version**: Calls `process::execute_all_ready()`
  - **Implementation**: Call new `sys_run_ready()` syscall
  - **Syscalls needed**: `sys_run_ready` ✅
  - **Notes**: Userspace now owns when tasks execute

- [x] `clear` command - Clear screen ✅ IMPLEMENTED
  - **Kernel version**: Calls `vga_buffer::clear_screen()`
  - **Implementation**: Call new `sys_clear_screen()` syscall
  - **Syscalls needed**: `sys_clear_screen` ✅

### 2.3 Enhance Existing Commands
- [ ] `spawn` - Reconcile with kernel semantics
  - **Current CLI**: `spawn <count>` (spawns N identical tasks)
  - **Current kernel**: `spawn <index>` (spawns specific test task)
  - **Decision needed**: Which model to use?
  - **Blocker**: None, but needs design decision

- [ ] `ps` - Add more process details
  - **Current**: PID and status only
  - **Enhanced**: Add task index, entry point, exit code
  - **Blocker**: May need `sys_get_process_state`

---

## Section 3: TTY/Display Management

These represent kernel code that needs exposure to userspace via syscalls.

### 3.1 Display Output Control
- [ ] Expose `vga_buffer` operations as syscalls
  - **Current**: Kernel-only, async terminal writes directly
  - **Target**: Userspace CLI uses syscalls for all output
  - **Syscalls**: `sys_write(1)` already does this ✅
  - **Status**: Mostly done, just need `sys_clear_screen`

### 3.2 Keyboard Input Handling
- [ ] Keep kernel terminal input reading (necessary for bootstrap)
  - **Current**: `task/terminal.rs` reads keyboard, queues to `input::add_input_char()`
  - **Change**: Queue to dedicated input buffer, not kernel shell
  - **Userspace**: Read via `sys_read(0)` from this buffer
  - **Status**: Already working for userspace ✅

### 3.3 TTY State Management (Optional, Phase 3+)
- [ ] Consider TTY abstraction (multiple terminals, sessions)
  - **Priority**: LOW - defer to Phase 3 or later
  - **Blocker**: None for basic migration

---

## Section 4: Process Management

Userspace should own process policy decisions.

### 4.1 Process Lifecycle
- [ ] Verify `sys_task_create` works from userspace
  - **Current status**: Implemented in syscall.rs ✅
  - **Test**: Call from CLI spawn command
  - **Blocker**: None

- [ ] Verify `sys_task_wait` works from userspace
  - **Current status**: Implemented in syscall.rs ✅
  - **Test**: Call from CLI after spawn
  - **Blocker**: None

### 4.2 Process Scheduling Decisions
- [ ] Move `execute_all_ready()` policy to userspace
  - **Current**: Kernel shell's `run` command calls this
  - **Target**: Userspace CLI calls `sys_run_ready()` syscall
  - **Implementation**: Add `sys_run_ready` syscall (see 1.2)
  - **Blocker**: `sys_run_ready` syscall needed

- [ ] Consider supervisor/task manager process
  - **Priority**: MEDIUM - future enhancement
  - **Notes**: Separate daemon to manage long-lived tasks
  - **Blocker**: None for initial migration

---

## Section 5: Kernel Cleanup

Remove policy from kernel once userspace takes over.

### 5.1 Shell Removal
- [ ] Remove `kernel/src/shell.rs`
  - **When**: After userspace CLI is complete
  - **Risk**: Low - no other code depends on it
  - **Test**: Ensure kernel still boots ✅

- [ ] Remove shell from kernel task startup
  - **Current**: `kernel/src/main.rs` spawns terminal task
  - **Change**: Terminal just reads input, doesn't process commands
  - **Location**: `kernel/src/task/terminal.rs` - remove shell instantiation
  - **Blocker**: None

### 5.2 Terminal Refactor
- [ ] Refactor `kernel/src/task/terminal.rs`
  - **Remove**: Shell instantiation, command execution
  - **Keep**: Keyboard input reading, forwarding to userspace via input buffer
  - **Keep**: VGA output via syscalls from userspace
  - **Result**: Minimal kernel terminal (just I/O plumbing)

### 5.3 Input Module Cleanup
- [ ] Ensure `kernel/src/input.rs` is userspace-ready
  - **Current**: Reads from keyboard, stores in buffer
  - **Change**: Ensure `sys_read(0)` properly reads from this buffer
  - **Test**: Verify userspace can read keyboard input
  - **Blocker**: Already working ✅

---

## Section 6: Boot Sequence

How does userspace CLI launch on startup?

### 6.1 Current Boot Flow
```
Kernel starts
  → Initialize hardware (GDT, paging, heap, interrupts)
  → Spawn terminal task (async executor)
  → Terminal reads keyboard & processes commands (shell.rs)
```

### 6.2 New Boot Flow (Target)
```
Kernel starts
  → Initialize hardware (GDT, paging, heap, interrupts)
  → Spawn terminal task (minimal, just input/output plumbing)
  → Spawn userspace CLI task with syscall capability
  → CLI loop: read input via sys_read → execute command → output via sys_write
  → Kernel never calls shell.rs
```

### 6.3 Implementation Tasks
- [ ] Modify `kernel/src/main.rs` boot sequence
  - **Change**: After executor setup, create userspace CLI task
  - **Blocker**: Need way to link userspace binary into kernel
  - **Note**: May need to embed CLI as kernel resource or symlink

- [ ] Ensure CLI task launches with correct entry point
  - **Current**: Kernel spawns test tasks via index
  - **Target**: Kernel spawns CLI as privileged task
  - **Blocker**: Task creation syscalls need review

- [ ] Remove explicit terminal task startup
  - **Current**: Terminal spawned and runs independently
  - **Change**: Terminal only needed for bootstrap; CLI takes over
  - **Optional**: Keep minimal terminal as fallback

### 6.4 Alternative: Init Process Approach
- [ ] Consider implementing proper init/systemd-like process
  - **Priority**: MEDIUM - Phase 3+ enhancement
  - **Benefit**: Clean separation, proper process management
  - **Blocker**: None for initial migration

---

## Section 7: Testing & Validation

Each change must be verified.

### 7.1 Syscall Testing
- [ ] Test `sys_clear_screen` from userspace
  - **Test case**: CLI `clear` command clears VGA buffer
  - **Validation**: Screen goes blank, prompt returns

- [ ] Test `sys_run_ready` from userspace
  - **Test case**: CLI `run` command executes ready tasks
  - **Validation**: Tasks run, exit codes captured

- [ ] Test all existing syscalls still work
  - **Current**: sys_read, sys_write, sys_task_create, sys_task_wait, sys_ps, sys_uptime
  - **Validation**: Existing CLI commands work without regression

### 7.2 Command Testing
- [ ] CLI `ping` - Returns "pong"
- [ ] CLI `clear` - Clears screen
- [ ] CLI `run` - Executes tasks
- [ ] CLI `spawn` - Creates tasks
- [ ] CLI `ps` - Lists processes
- [ ] CLI `echo` - Echoes text
- [ ] CLI `help` - Shows commands
- [ ] CLI `exit` - Exits cleanly

### 7.3 Boot Testing
- [ ] Kernel boots without shell.rs
- [ ] Userspace CLI launches on boot
- [ ] CLI accepts input and processes commands
- [ ] No kernel panics or crashes
- [ ] All commands responsive

### 7.4 Integration Testing
- [ ] Build succeeds (zero errors, zero warnings)
- [ ] Bootimage generation succeeds
- [ ] QEMU launches without crashes
- [ ] CLI fully functional
- [ ] No functionality loss from Phase 2

---

## Section 8: Blockers & Dependencies

Must resolve these before Phase 3.

### Critical Blockers (MUST FIX)
- [x] **Blocker #1**: Add `sys_clear_screen` syscall ✅ DONE
  - **Impact**: `clear` command, terminal management
  - **Resolution**: 30 mins - expose existing vga_buffer function
  - **Dependency**: None

- [x] **Blocker #2**: Add `sys_run_ready` syscall ✅ DONE
  - **Impact**: `run` command, batch task execution
  - **Resolution**: 30 mins - expose existing process function
  - **Dependency**: None

- [ ] **Blocker #3**: Ensure userspace CLI compiles and runs
  - **Impact**: All policy operations
  - **Resolution**: 1-2 hours - add commands, test syscalls
  - **Dependency**: Blockers #1 & #2 ✅

### Medium Blockers (SHOULD FIX)
- [ ] **Blocker #4**: Boot sequence launches userspace CLI
  - **Impact**: Clean architecture on startup
  - **Resolution**: 1 hour - modify main.rs, create init task
  - **Dependency**: Blocker #3

### Low Blockers (NICE TO FIX)
- [ ] **Blocker #5**: Remove kernel shell.rs
  - **Impact**: Code cleanliness
  - **Resolution**: 15 mins - delete file, update lib.rs
  - **Dependency**: Blocker #4

---

## Section 9: Implementation Order

Do in this sequence to minimize risk.

```
✅ 1. Add sys_clear_screen syscall - DONE
✅ 2. Add sys_run_ready syscall - DONE
✅ 3. Add missing CLI commands (ping, clear, run) - DONE
⏳ 4. Test all CLI commands work - IN PROGRESS
5. Update boot sequence (optional, can defer)
6. Remove kernel shell.rs (optional, can defer)
7. Comprehensive integration testing
8. Clean build & documentation
```

---

## Section 10: Exit Criteria

Before moving to Phase 3, ALL of these must be true:

- [ ] All 7 shell commands work identically in userspace CLI
- [ ] No regression from Phase 2 functionality
- [ ] Kernel has no shell.rs (or it's unused)
- [ ] All syscalls tested and working
- [ ] Build: Zero errors, zero warnings
- [ ] Bootimage generated successfully
- [ ] QEMU boots and CLI is responsive
- [ ] Clear architectural separation (kernel = mechanism, userspace = policy)
- [ ] Documentation updated
- [ ] Ready to start Phase 3 preemptive multitasking

---

## Timeline Estimate

| Task | Effort | Duration |
|------|--------|----------|
| Add syscalls (clear, run_ready) | Small | 1 hour |
| Add CLI commands (ping, clear, run) | Small | 1 hour |
| Test all commands | Small | 1 hour |
| Boot sequence refactor | Medium | 1 hour |
| Remove shell.rs | Small | 0.5 hour |
| Integration testing | Medium | 1 hour |
| Documentation | Small | 1 hour |
| **TOTAL** | | **~6-7 hours** |

---

## Notes for Phase 3

Once this migration is complete:

✅ Kernel is minimal (pure mechanism)  
✅ Userspace policy is clean and isolated  
✅ Foundation is solid for preemptive multitasking  
✅ No architectural debt blocking Phase 3  
✅ Can safely add timer interrupts, preemption  

Phase 3 can focus on:
- Cooperative/preemptive scheduling
- Timer interrupt handling
- Context switching optimizations
- No policy layer confusion

---

## Revision History

| Date | Version | Status |
|------|---------|--------|
| 2026-01-17 | 1.0 | Initial checklist |
| 2026-01-18 | 1.1 | Critical blockers resolved - syscalls implemented and CLI commands added |


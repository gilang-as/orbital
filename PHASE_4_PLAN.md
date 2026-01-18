# Phase 4: Full Userspace Shell Migration - Implementation Plan

**Status**: Planned (Ready to start)  
**Dependency**: Phase 3 Complete ✅  
**Estimated Duration**: 3-4 hours  
**Target**: Userspace CLI runs with all commands via syscalls  

---

## Overview

Phase 4 transitions the shell completely from kernel to userspace. The kernel will embed the userspace CLI binary, load it on boot, and execute it as a userspace process. All shell commands will execute via syscalls instead of direct kernel calls.

**Architecture Target**:
```
Hardware Keyboard
         ↓
Terminal Task (Kernel) - Pure I/O [stays]
         ↓
Input Buffer
         ↓
Userspace CLI Process - Command Execution [new]
    ↓        ↓        ↓
syscall  syscall  syscall... (12 total)
    ↓        ↓        ↓
Kernel Syscall Handlers
         ↓
VGA Display / Process Management / etc
```

---

## Phase 4 Tasks

### Task 1: Build Userspace CLI Binary
**Objective**: Compile standalone userspace CLI that can run independently  
**Current State**: CLI source exists in `userspace/cli/src/main.rs`  
**Action Items**:
1. Build `userspace/cli` in release mode
2. Locate binary at `target/x86_64-unknown-linux-gnu/release/orbital-cli` (or similar)
3. Verify binary size and compatibility
4. Test standalone compilation

**Expected Outcome**: Standalone binary ready for embedding

### Task 2: Embed Binary in Kernel Build
**Objective**: Include userspace CLI binary as kernel resource  
**Approaches**:

**Option A: Build Script** (Recommended)
- Create `kernel/build.rs` build script
- Compile userspace/cli during kernel build
- Use `include_bytes!()` to embed binary
- Binary accessible as `static USERSPACE_CLI: &[u8] = include_bytes!(...)`

**Option B: Cargo Features**
- Use conditional compilation
- Enable feature to include CLI binary
- Useful if binary is large or optional

**Option C: Runtime Loading** (Future)
- Load from filesystem at runtime
- Requires filesystem implementation
- Defer to Phase 5+

**Implementation Choice**: Option A - Build Script  
**Why**: Clean, automatic, no special handling needed

**Action Items**:
1. Create `kernel/build.rs` script
2. Add build-dependencies in `kernel/Cargo.toml` (if needed)
3. Use script to compile `userspace/cli`
4. Embed using `include_bytes!()`
5. Export binary slice from `binary_loader.rs`

**Expected Outcome**: Kernel build includes userspace CLI binary automatically

### Task 3: Update Boot Sequence
**Objective**: Load userspace CLI instead of spawning kernel shell task  
**Current Code** (kernel/src/main.rs):
```rust
executor.spawn(Task::new(orbital_kernel::task::terminal::terminal()));
executor.spawn(Task::new(orbital_kernel::task::cli::shell()));
```

**New Code**:
```rust
executor.spawn(Task::new(orbital_kernel::task::terminal::terminal()));
// Load userspace CLI binary
let cli_binary = include_bytes!("../target/.../orbital-cli");
let _ = orbital_kernel::binary_loader::execute_binary(cli_binary, "orbital-cli", &mut executor);
```

**Implementation**:
1. Modify `binary_loader.rs` `execute_binary()` to actually load binary
2. Create task wrapper that executes userspace binary
3. Update `kernel/src/main.rs` to call loader
4. Remove `executor.spawn(Task::new(orbital_kernel::task::cli::shell()));`

**Challenge**: Userspace binary execution
- Binary expects to run in userspace with syscalls
- Kernel needs to call it appropriately
- May need simple wrapper or syscall handler adjustment

**Expected Outcome**: Userspace CLI runs on boot, terminal I/O works

### Task 4: Verify Syscall Flow
**Objective**: Ensure all commands work via userspace syscalls  
**Testing**:
1. Boot system
2. Terminal displays prompt
3. Type `help` → syscall to kernel → displays command list
4. Type `echo hello` → syscall to kernel → displays "hello"
5. Type `ps` → syscall to kernel → lists processes
6. Type `spawn` → syscall to kernel → creates process
7. Type `exit` → syscall to kernel → terminates CLI

**Expected Behavior**:
- All 11 commands work
- Zero kernel errors
- Commands execute via syscalls (not direct calls)
- Clean separation maintained

### Task 5: Remove Phase 2.5 Kernel Code
**Objective**: Delete temporary kernel shell implementation  
**Files to Delete**:
- `kernel/src/task/cli.rs` - Kernel shell task
- `kernel/src/shell_commands.rs` - Temporary command implementations

**Files to Update**:
- `kernel/src/lib.rs` - Remove module exports
- `kernel/src/task/mod.rs` - Remove cli module declaration

**Result**:
- Kernel contains only mechanism (I/O, process management, syscalls)
- All policy moved to userspace
- Clean, minimal kernel implementation

### Task 6: Documentation & Verification
**Objective**: Document final architecture and verify all criteria  
**Actions**:
1. Update `PHASE_4_COMPLETION.md` with results
2. Create architecture diagrams showing userspace execution
3. Add syscall flow documentation
4. Verify all tests pass
5. Test in QEMU with real kernel boot

---

## Implementation Order

```
1. Build standalone userspace CLI binary
   ↓
2. Create kernel build script (build.rs)
   ↓
3. Embed binary in kernel via include_bytes!()
   ↓
4. Update binary_loader.rs to actually execute binary
   ↓
5. Update kernel/src/main.rs boot sequence
   ↓
6. Test: boot, run commands via syscalls
   ↓
7. Delete kernel shell task (cli.rs)
   ↓
8. Delete kernel commands (shell_commands.rs)
   ↓
9. Clean up lib.rs, task/mod.rs
   ↓
10. Final testing and documentation
```

---

## Potential Challenges & Solutions

### Challenge 1: Userspace Binary Execution Model
**Problem**: Userspace binary compiled as `#![no_std]` expects syscalls, but kernel needs to call it  
**Solution**:
- Create simple task wrapper that jumps to binary entry point
- Syscalls from userspace return to kernel handler
- Kernel provides all I/O via syscalls (sys_read, sys_write)
- Clean boundary maintained via ABI

### Challenge 2: Input Handling
**Problem**: Terminal currently queues input to kernel shell task  
**Solution**:
- Terminal continues to queue to input buffer (unchanged)
- Userspace CLI task reads from buffer via `sys_read()` syscall
- Same decoupling mechanism, different reader

### Challenge 3: Memory Layout
**Problem**: Userspace binary needs memory where kernel can find it  
**Solution**:
- Binary embedded in kernel (known address)
- Kernel loads into process memory on boot
- ELF or raw binary handling (start simple with raw)

### Challenge 4: Exit Handling
**Problem**: Userspace CLI exits, but keyboard still generates input  
**Solution**:
- CLI exit calls `sys_exit()` syscall
- Kernel marks process as exited
- Restart CLI on next boot (future: respawn on exit)

---

## Files Involved

**To Create**:
- `kernel/build.rs` - Build script to compile and embed CLI binary

**To Modify**:
- `kernel/src/binary_loader.rs` - Implement actual binary execution
- `kernel/src/main.rs` - Update boot sequence
- `kernel/src/lib.rs` - Remove shell_commands export
- `kernel/src/task/mod.rs` - Remove cli module

**To Delete**:
- `kernel/src/task/cli.rs` - Kernel shell task
- `kernel/src/shell_commands.rs` - Temporary commands

**To Document**:
- `PHASE_4_COMPLETION.md` - Final results

---

## Success Criteria

- ✅ Userspace CLI binary compiles
- ✅ Binary embedded in kernel automatically via build.rs
- ✅ Kernel boots with embedded binary
- ✅ Terminal reads input, queues to buffer
- ✅ Userspace CLI reads from buffer via syscalls
- ✅ All 11 commands work via syscalls
- ✅ Commands produce correct output to VGA display
- ✅ Kernel shell task removed from code
- ✅ Zero build errors
- ✅ Documentation complete

---

## Testing Plan

### Unit Tests
```bash
cargo test --lib binary_loader  # Test loader logic
cargo test --lib process        # Test process creation
cargo test --lib syscall        # Test syscall handlers
```

### Integration Tests
```bash
cargo bootimage                 # Build bootimage with embedded CLI
# Boot in QEMU and test manually
```

### Manual Testing
1. **Boot Test**: System boots, prompt appears
2. **help**: Lists all 11 commands
3. **echo test**: Prints "test"
4. **ps**: Shows processes (terminal, CLI, maybe others)
5. **pid**: Shows CLI process ID
6. **uptime**: Displays kernel uptime
7. **spawn test 4**: Creates new process
8. **wait <pid>**: Waits for process
9. **clear**: Clears screen
10. **exit**: Terminates CLI gracefully

---

## Rollback Plan

If Phase 4 causes issues:
1. Revert to Phase 3 commit
2. Keep kernel shell task functional
3. Binary loader infrastructure remains for Phase 5
4. No data loss (git preserves all versions)

---

## Phase 4 Blockers

None identified. Phase 3 infrastructure is complete:
- ✅ Binary loader module ready
- ✅ Process extensions implemented
- ✅ Syscalls stable
- ✅ Build system clean

---

## Expected Timeline

| Task | Duration | Start | End |
|------|----------|-------|-----|
| Build CLI binary | 15 min | now | +15m |
| Create build.rs | 30 min | +15m | +45m |
| Embed binary | 20 min | +45m | +65m |
| Implement execution | 45 min | +65m | +110m |
| Test & debug | 45 min | +110m | +155m |
| Remove kernel code | 20 min | +155m | +175m |
| Documentation | 30 min | +175m | +205m |
| **Total** | **~3.5 hours** | | |

---

## Next Steps After Phase 4

Once Phase 4 complete:

1. **Phase 5: Advanced Userspace Features**
   - ELF binary loader support
   - Multiple userspace processes
   - Complex IPC patterns

2. **Phase 6: Preemptive Multitasking**
   - Timer-based context switches
   - Process priority scheduling

3. **Phase 7: Memory Protection**
   - Page tables for userspace isolation
   - Virtual memory management

---

## Phase 4 Ready Status

✅ **Architecture Proven**: Phase 3 validated design  
✅ **Infrastructure Ready**: Binary loader and process extensions in place  
✅ **Build System Clean**: No blockers identified  
✅ **Syscalls Stable**: All 12 working and tested  
✅ **Userspace CLI Ready**: Compiles successfully with all features  

**Ready to begin Phase 4 implementation!**

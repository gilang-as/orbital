# Orbital OS - Shell Commands Reference

**Purpose**: Reference for all interactive shell commands
**Scope**: 7 implemented commands in userspace shell
**Last Verified**: January 2026
**Implementation Status**: IMPLEMENTED

---

## Command Summary

| Command | Arguments | Description |
|---------|-----------|-------------|
| `help` | - | Display available commands |
| `echo` | `<text>` | Print text to output |
| `pid` | - | Show current process ID |
| `uptime` | - | Show kernel uptime |
| `ps` | - | List all processes |
| `clear` | - | Clear the screen |
| `exit` | - | Terminate shell |

---

## Command Details

### help

**Syntax**: `help`

**Description**: Display list of available commands with brief descriptions.

**Output**:
```
[Phase 9] Available Commands:
  help         - Show this help
  echo <text>  - Echo text
  pid          - Show current PID
  uptime       - Show kernel uptime
  ps           - List processes
  clear        - Clear screen
  exit         - Exit shell
```

**Syscalls Used**: `sys_write` (#2)

**Implementation**: `userspace/minimal/src/main.rs`

---

### echo

**Syntax**: `echo <text>`

**Description**: Print the specified text to standard output.

**Examples**:
```
shell> echo Hello World
Hello World

shell> echo Test message here
Test message here
```

**Behavior**:
- Prints everything after "echo " prefix
- Adds newline after output
- Trims leading/trailing whitespace from command

**Syscalls Used**: `sys_write` (#2)

---

### pid

**Syntax**: `pid`

**Description**: Display the process ID of the current shell instance.

**Output**:
```
shell> pid
PID: 1
```

**Values**: 1, 2, or 3 (three concurrent shells)

**Syscalls Used**: `sys_getpid` (#12), `sys_write` (#2)

---

### uptime

**Syntax**: `uptime`

**Description**: Display kernel uptime since boot.

**Output Format**: `Uptime: Xm Ys`

**Examples**:
```
shell> uptime
Uptime: 0m 5s

shell> uptime
Uptime: 2m 30s

shell> uptime
Uptime: 15m 42s
```

**Behavior**:
- Retrieves seconds from kernel via syscall
- Converts to minutes and seconds
- Uses stack-based integer conversion (no heap)

**Syscalls Used**: `sys_uptime` (#9), `sys_write` (#2)

**Accuracy**: ~10ms resolution (depends on timer frequency)

---

### ps

**Syntax**: `ps`

**Description**: List all processes with their status.

**Output Format**:
```
PID Status
  1 Running
  2 Running
  3 Running
```

**Status Values**:
| Status | Meaning |
|--------|---------|
| Ready | Waiting to be scheduled |
| Running | Currently executing |
| Blocked | Waiting for event |
| Exited(N) | Terminated with code N |

**Behavior**:
- Kernel formats process list into buffer
- Shell receives and displays formatted string
- Uses 512-byte stack buffer

**Syscalls Used**: `sys_ps` (#8), `sys_write` (#2)

---

### clear

**Syntax**: `clear`

**Description**: Clear the VGA text display.

**Behavior**:
- Fills screen with spaces
- Resets cursor to top-left
- Does not clear scrollback (none exists)

**Syscalls Used**: `sys_clear_screen` (#10)

---

### exit

**Syntax**: `exit`

**Description**: Terminate the current shell process.

**Output**:
```
shell> exit
[Phase 9] Shell exiting
```

**Behavior**:
- Prints exit message
- Calls `sys_exit(0)`
- Process status changes to `Exited(0)`
- Does not return

**Syscalls Used**: `sys_write` (#2), `sys_exit` (#3)

**Note**: With 3 concurrent shells, exiting one does not affect others.

---

## Input Handling

### Line Input

- **Buffer Size**: 256 bytes maximum
- **Terminator**: Enter key (newline character)
- **Reading**: Non-blocking via `sys_read`

### Parsing

- Leading/trailing whitespace trimmed
- Command matched by prefix ("echo" matches "echo hello")
- Unknown commands display error message

---

## Error Handling

### Unknown Command

```
shell> foo
[Phase 9] Unknown command: foo
```

### Empty Input

Empty lines (just Enter) are ignored silently.

---

## Implementation Notes

### No Heap Allocation

The shell operates entirely on stack:
- Input buffer: 256 bytes on stack
- PS buffer: 512 bytes on stack
- Integer conversion: 20-byte stack buffer

### Syscall Wrapper

```rust
fn syscall(nr: i64, a1: i64, a2: i64, a3: i64) -> i64 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inout("rax") nr => result,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            out("rcx") _,
            out("r11") _,
            clobber_abi("C"),
        );
    }
    result
}
```

### Command Dispatch

```rust
fn execute_command(cmd: &str) {
    let trimmed = cmd.trim();

    if trimmed == "help" {
        // show help
    } else if trimmed.starts_with("echo ") {
        // echo text
    } else if trimmed == "pid" {
        // show pid
    } else if trimmed == "uptime" {
        // show uptime
    } else if trimmed == "ps" {
        // list processes
    } else if trimmed == "clear" {
        // clear screen
    } else if trimmed == "exit" {
        // exit shell
    } else {
        // unknown command
    }
}
```

---

## Future Commands (Not Implemented)

| Command | Purpose | Phase |
|---------|---------|-------|
| `kill <pid>` | Terminate process | 12+ |
| `spawn <cmd>` | Start new process | 12+ |
| `cat <file>` | Display file | 10+ |
| `ls` | List files | 10+ |
| `cd <dir>` | Change directory | 10+ |

---

**Document Status**: COMPLETE
**Commands Documented**: 7 of 7

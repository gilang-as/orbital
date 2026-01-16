# Orbital CLI - Userspace Command Dispatcher

## Overview

The Orbital CLI is a **minimal command-line interface** running entirely in userspace. It demonstrates the kernel's policy-free design by implementing all command logic, argument parsing, and output formatting outside the kernel.

## Purpose

### Problem Solved

Early Unix kernels often embedded command interpreters (shell logic) in the kernel, leading to:
- Bloated kernel code
- Policy decisions in privileged mode
- Difficult to extend or modify commands
- Security implications of complex kernel logic

### Solution

Orbital CLI moves **all command logic to userspace**:

```
Userspace CLI (this program)
    ↓ parses input
    ↓ executes command
    ↓ formats output
    ↓ calls sys_write
Kernel
    ↓ validates fd, ptr, len
    ↓ copies buffer
Kernel TTY
    ↓ routes to serial
Hardware (serial port)
```

Benefits:
- **Kernel stays minimal** — No command parsing, logic, or formatting
- **Policy in userspace** — Easy to modify, test, and extend
- **Secure by design** — Complex logic doesn't run in privileged mode
- **Flexible** — Multiple CLIs could coexist with different commands
- **Clear separation** — Kernel handles mechanism, userspace handles policy

## Architecture

### Binary: `userspace/cli/src/main.rs`

**Entry Point:**
```rust
fn main() {
    // Print welcome banner
    // Execute hardcoded demo commands
    // Show usage
}
```

**Command Dispatcher:**
```rust
struct CommandDispatcher;

impl CommandDispatcher {
    fn execute(&self, input: &str) { ... }  // Parse and dispatch
    fn cmd_help(&self) { ... }               // help command
    fn cmd_echo(&self, args: &[&str]) { ... } // echo command
    fn cmd_unknown(&self, cmd: &str) { ... } // Unknown command
}
```

### Data Flow

```
Input String
    ↓
execute() → trim, split whitespace
    ↓
Match command name
    ├→ "help" → cmd_help()
    │           ├→ build help text
    │           ├→ sys_write(fd=1, text, len)
    │           └→ output help on stdout
    │
    ├→ "echo" → cmd_echo(args)
    │           ├→ join args with spaces
    │           ├→ sys_write(fd=1, text, len)
    │           └→ output on stdout
    │
    └→ other → cmd_unknown(cmd)
                ├→ build error message
                ├→ sys_write(fd=1, msg, len)
                └→ output error on stdout
```

## Implemented Commands

### help

**Purpose:** Display available commands

**Usage:** `help`

**Output:**
```
Orbital CLI - Available Commands:

  help              Show this help message
  echo <text>       Echo text to stdout
```

**Implementation:**
- No arguments
- Prints formatted help text via three `sys_write()` calls
- Each call writes one line to maintain small buffer sizes

### echo

**Purpose:** Print text to stdout

**Usage:** `echo <text>`

**Behavior:**
- With args: Echoes space-separated arguments followed by newline
- Without args: Prints newline only
- No interpretation of escape sequences (e.g., `\n` printed literally)

**Examples:**
```
> echo Hello
Hello

> echo Hello World
Hello World

> echo
<blank line>
```

**Implementation:**
- Reconstruct argument string with spaces
- Add trailing newline
- Call `sys_write(fd=1, text.as_ptr(), text.len())`

### Unknown Command Handler

**Purpose:** Inform user of invalid commands

**Output:** `unknown command: <cmd_name>`

**Implementation:**
- Write prefix via sys_write
- Write command name via sys_write
- Write newline via sys_write

## Design Constraints

### What CLI Does

✓ Parse hardcoded command strings
✓ Dispatch to command handlers
✓ Format output text
✓ Call sys_write for all output
✓ Demonstrate userspace policy

### What CLI Does NOT Do

✗ Read from stdin (blocked on sys_read implementation)
✗ Shell features (pipes, redirection, wildcards)
✗ Variable expansion or substitution
✗ Scripting or control flow
✗ Filesystem access
✗ Environment variables
✗ Process management

## Current Limitations

### Hardcoded Commands

Today's CLI executes a fixed sequence of commands:
```rust
let commands = vec![
    "help",
    "echo Hello from Orbital",
    "echo Userspace policy in action",
];
```

This is a **demonstration mode**. In a full implementation:
1. Print prompt: `> `
2. Read user input via `sys_read()` (not yet implemented)
3. Execute command
4. Repeat

### No Interactive Loop

Since `sys_read()` isn't implemented, the CLI runs a hardcoded sequence then exits. This is sufficient to prove:
- Command parsing works
- Command dispatch works
- Userspace output via syscalls works
- Kernel doesn't interpret command content

### Buffer Size Constraint

Each `sys_write()` call is limited to 4096 bytes (matching kernel validation). The CLI handles this by:
- Writing small strings (command output, help text, prompts)
- Multiple `sys_write()` calls if needed (each line separately)

## Code Organization

### File Structure

```
userspace/cli/
├── Cargo.toml           # Package manifest
└── src/
    └── main.rs          # CLI implementation (92 lines)
```

### Dependencies

- `orbital-ipc` — Provides `syscall_write(fd, ptr, len)` wrapper
- `orbital-common` — Shared types (currently unused)

### Compilation

```bash
cd userspace/cli
cargo build --release
```

Output: `target/release/orbital-cli`

## Integration with Kernel

### Syscall Dependencies

CLI uses only **`sys_write(2)`** syscall:
- `sys_write(fd=1, ptr, len)` — Write to stdout
- FD value 1 means stdout (no stderr used by CLI)
- Returns bytes written or error code

### No Kernel Assumptions

CLI doesn't assume:
- Specific kernel architecture
- Process IDs or task management
- Memory layout or permissions
- Signal handling
- File descriptors beyond fd=1

The CLI works with just the sys_write primitive.

## Safety Properties

### Userspace Privileges

CLI runs in user mode (not kernel mode). Safety implications:
- Can't directly access hardware
- Can't modify kernel structures
- Isolated from other programs (future)
- Malicious input can't harm kernel
- Bugs in CLI don't crash kernel

### Syscall Safety

Each `sys_write()` call is validated by kernel:
- FD checked (only 1, 2 allowed)
- Pointer validated (not NULL)
- Length checked (max 4096 bytes)
- Buffer safely copied to kernel
- Kernel can't be overflowed

### Input Parsing Safety

CLI doesn't:
- Use unsafe code (all safe Rust)
- Buffer unbounded input
- Interpret binary data
- Parse complex formats

## Future Enhancements

### Phase 1: Interactive Input

Implement `sys_read()` in kernel, then:
```rust
fn read_command() -> String {
    let mut input = String::new();
    let _ = sys_read(0, input.as_mut_ptr(), 512)?; // stdin = fd 0
    input.trim().to_string()
}

loop {
    print_prompt();
    let cmd = read_command();
    dispatcher.execute(&cmd);
}
```

### Phase 2: More Commands

Expand command set:
- `cat <file>` — Read files (requires sys_open, sys_read)
- `ls` — List files (requires directory syscalls)
- `mkdir <name>` — Create directory
- `pwd` — Print working directory
- `cd <path>` — Change directory
- `time` — Show system time (requires sys_time)

### Phase 3: Shell Features

Implement shell-like behavior:
- Environment variables: `name=value echo $name`
- Pipes: `echo hello | cat`
- Redirects: `echo text > file`
- Wildcards: `ls *.txt`
- Background jobs: `sleep 10 &`

### Phase 4: Script Support

Load and execute command scripts:
- `exec script.sh`
- Parse shell syntax
- Control flow (if, for, while)

### Phase 5: Multi-User

- Login system with credentials
- Per-user command history
- User isolation (future when process model exists)

## Comparison: Kernel vs Userspace

| Responsibility | Kernel | Userspace CLI |
|---|---|---|
| **Syscall dispatch** | ✓ Routes syscalls | × |
| **Command parsing** | × | ✓ Splits args |
| **Output formatting** | × | ✓ Builds strings |
| **Memory management** | ✓ Validates pointers | × (trusts kernel) |
| **Error reporting** | × | ✓ "unknown command" |
| **Help text** | × | ✓ help command |
| **Extension** | Very hard (recompile kernel) | Easy (modify CLI) |

## Testing

To test the CLI (once syscall entry point is wired):

```bash
# Build CLI
cd userspace/cli
cargo build --release

# Copy to disk image or run in userspace environment
# Expected output:
# > Orbital CLI v0.1.0
# > Type 'help' for available commands.
# > > help
# > Orbital CLI - Available Commands:
# > ...
# > > echo Hello from Orbital
# > Hello from Orbital
# > Goodbye!
```

## Summary

The Orbital CLI demonstrates a **minimal, policy-free kernel** by moving all command logic to userspace:

- **Kernel:** Validates fd, ptr, len; copies buffer; routes to TTY
- **Userspace CLI:** Parses input; dispatches commands; formats output

This separation is the foundation of Unix design philosophy: kernel provides mechanisms, userspace provides policy. The CLI can be replaced, extended, or modified without touching kernel code.

## References

- [Kernel Syscall Design](../13.%20Syscall%20Skeleton%20Design.md)
- [sys_write Specification](../syscalls/sys_write.md)
- [TTY Device Primitive](../kernel/tty.md)
- [IPC Wrapper Functions](../../../userspace/ipc/src/lib.rs)

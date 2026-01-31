# Orbital OS – Syscall & IPC Boundary Specification

> **Purpose**
>
> This document defines the **contract between kernel and userspace** in Orbital OS. It specifies:
> - What the kernel MUST do
> - What the kernel MUST NOT do
> - How userspace communicates with the kernel
> - Error handling and failure semantics
> - Versioning and compatibility guarantees
>
> **This is a specification, not an implementation.** It defines the boundary, not the code.

---

## 1. Kernel Responsibilities (Strictly Limited)

The kernel provides **only primitives**, not policy. All system logic is in userspace.

### 1.1 Memory & Virtual Address Space

**MUST**:
- Manage virtual address spaces per process
- Enforce memory isolation between processes
- Provide page allocation APIs
- Implement copy-on-write for fork (if fork exists)
- Prevent use-after-free via capability-based access
- Support zero-copy buffer registration

**MUST NOT**:
- Make decisions about memory quotas per service
- Implement service-level OOM policies
- Manage memory limits (userspace does this via syscalls)

### 1.2 Task & Process Management

**MUST**:
- Create and destroy tasks/processes
- Manage task queues and scheduling
- Enforce CPU time accounting
- Support task spawning from userspace
- Track task state (created, running, blocked, exited)
- Provide task IDs and capability handles

**MUST NOT**:
- Decide which service runs when (scheduling policy is userspace)
- Manage service restart policies
- Implement service supervision

### 1.3 Interrupt & Exception Handling

**MUST**:
- Handle CPU interrupts and exceptions
- Route hardware events to appropriate handlers
- Provide timer interrupt infrastructure
- Provide exception safety boundaries

**MUST NOT**:
- Perform business logic in interrupt handlers
- Block in exception handling code
- Make decisions about device handling (userspace drivers do this)

### 1.4 Inter-Process Communication (IPC)

**MUST**:
- Provide message passing primitives
- Enforce access control (who can send to whom)
- Prevent message tampering across trust boundaries
- Support capability-based addressing
- Guarantee message atomicity and ordering per channel
- Provide IPC versioning metadata

**MUST NOT**:
- Route or broker messages (userspace IPC router does this)
- Enforce IPC protocols or semantics
- Buffer unlimited messages (userspace manages backpressure)

### 1.5 Core Security Primitives

**MUST**:
- Enforce capability-based access control
- Manage capability delegation
- Prevent cross-process register/memory access
- Audit security-sensitive syscalls
- Support mandatory access control (MAC) via labels

**MUST NOT**:
- Implement policy decisions (role-based access, service permissions)
- Store security policies (userspace policy engine does this)
- Make deny/allow decisions beyond capability enforcement

### 1.6 Hardware Abstraction

**MUST**:
- Provide access to memory-mapped I/O with safety checks
- Route interrupts to registered handlers
- Provide low-level driver APIs
- Manage interrupt masking/unmasking

**MUST NOT**:
- Implement device drivers
- Make decisions about device ownership
- Implement protocol-level handling (that's userspace networking)

---

## 2. Userspace Responsibilities

All logic **not strictly needed in the kernel** is in userspace.

### 2.1 Management Daemon (`managementd`)

**Responsibilities**:
- Single point of system state mutation
- Accept commands from CLI/API
- Translate commands to kernel syscalls
- Enforce RBAC policies
- Manage service lifecycle
- Implement restart policies
- Handle authentication/authorization
- Audit all state-changing operations

**Interface**:
- IPC API: Accepts commands from CLI and packages
- Kernel syscalls: Communicates with kernel
- Process supervisor: Manages child processes

### 2.2 IPC Router

**Responsibilities**:
- Route messages between services
- Enforce IPC access control lists (ACLs)
- Version negotiation for IPC endpoints
- Message logging and observability
- Handle IPC timeouts and failures

**Contract**: No service-to-service direct trust. All IPC flows through router.

### 2.3 Service Supervisor

**Responsibilities**:
- Monitor service health
- Implement restart policies
- Isolate crashes
- Manage process groups
- Report service status

### 2.4 Device Drivers (Userspace Drivers)

**Responsibilities**:
- Register interrupt handlers
- Manage device-specific protocols
- Implement hardware abstraction
- Handle device failures gracefully
- Signal errors to management daemon

### 2.5 CLI & API Server

**Responsibilities**:
- Parse user commands
- Validate input
- Translate to management daemon requests
- Format responses
- Handle API versioning

---

## 3. FORBIDDEN in Kernel

These MUST NEVER be implemented in the kernel:

| What | Why | Where It Goes |
|------|-----|---|
| Service restart policies | Policy is userspace | Supervisor |
| Memory quotas per service | Quota enforcement is userspace | Managementd |
| IPC routing/brokering | Messages flow through IPC router | IPC Router service |
| Authentication/authorization | Policy belongs in userspace | Managementd |
| Device drivers | No state machines in kernel | Userspace drivers |
| Configuration parsing | No business logic | Managementd |
| Service discovery | Dynamic state is userspace | Discovery service |
| Scheduling policy | Policy belongs in userspace | Scheduler policy module |
| Logging/audit policy | Configuration is userspace | Audit service |
| Network protocol handling | That's a feature | Networking packages |
| Package management | That's a feature | Package manager service |
| Backup/restore logic | That's a feature | Recovery service |

---

## 4. Syscall Categories & Contracts

### 4.1 Task/Process Syscalls

```
Category: task
Prefix:   task_*
```

**Syscalls**:

| Syscall | Args | Returns | Preconditions | Postconditions |
|---------|------|---------|----------------|----------------|
| `task_create(entry, stack_base, stack_size, caps)` | entry point, memory, capability set | task_id or error | Caller has CAP_TASK_CREATE | New task created, not scheduled |
| `task_exit(exit_code)` | exit code | never returns | Task is running | Task stopped, resources freed |
| `task_get_id()` | none | task_id | Task exists | Returns numeric ID |
| `task_set_affinity(task_id, cpu_mask)` | task handle, CPU mask | success or error | Caller has CAP_SCHED | Task may migrate |
| `task_wait(task_id, timeout)` | task handle, optional timeout_ms | exit_code or TIMEOUT | Task exists | Waits for task death |

**Error Handling**:
- `ENOEXIST`: Task/capability doesn't exist
- `EACCES`: Caller lacks capability
- `EINVAL`: Invalid arguments
- `ETIMEOUT`: Timeout waiting for task
- `EBADF`: Invalid task handle

---

### 4.2 Memory Syscalls

```
Category: memory
Prefix:   mem_*
```

**Syscalls**:

| Syscall | Args | Returns | Preconditions | Postconditions |
|---------|------|---------|----------------|----------------|
| `mem_alloc(size, flags)` | bytes, alignment flags | virtual_address or error | Size > 0 | Memory mapped, readable/writable |
| `mem_free(address, size)` | virtual address, size | success or error | Address is valid, mapped | Memory unmapped, inaccessible |
| `mem_protect(address, size, perms)` | address, size, R/W/X flags | success or error | Address is mapped | New permissions applied |
| `mem_share(address, size, target_task, caps)` | address, target task, capability | success or error | Both tasks exist, memory mapped | Target task gains access capability |
| `mem_get_stats()` | none | memory_stats struct | None | Returns usage info (page count, etc) |

**Error Handling**:
- `ENOMEM`: Allocation failed
- `ENOMAP`: Address not mapped
- `EACCES`: Permission denied
- `EINVAL`: Invalid address range

---

### 4.3 IPC Syscalls

```
Category: ipc
Prefix:   ipc_*
```

**Syscalls**:

| Syscall | Args | Returns | Preconditions | Postconditions |
|---------|------|---------|----------------|----------------|
| `ipc_endpoint_create(flags)` | endpoint flags | endpoint_handle or error | Caller has CAP_IPC | Endpoint created, receives msgs |
| `ipc_endpoint_destroy(endpoint)` | endpoint handle | success or error | Endpoint exists, task owns it | Endpoint closed, msgs dropped |
| `ipc_send(endpoint, message, timeout)` | target endpoint, msg bytes, timeout_ms | bytes_sent or error | Endpoint exists, sender has access | Message queued at target |
| `ipc_receive(endpoint, max_size, timeout)` | endpoint, buffer size, timeout_ms | (message_bytes, sender_cap) or error | Endpoint exists, task owns it | Message delivered or TIMEOUT |
| `ipc_send_cap(endpoint, cap_handle)` | endpoint, capability | success or error | Cap is valid, delegable | Receiver gains capability |
| `ipc_get_version(endpoint)` | endpoint | version_u32 | Endpoint exists | Returns IPC version |

**Contract**:
- Messages are fixed-size or length-prefixed
- Sender identity is always available
- Capabilities can only be delegated by owner
- Message ordering is FIFO per endpoint

**Error Handling**:
- `ENOEXIST`: Endpoint doesn't exist
- `EACCES`: No access to endpoint
- `ETIMEOUT`: Send or receive timeout
- `EMSGSIZE`: Message too large
- `EBADF`: Invalid handle

---

### 4.4 Time Syscalls

```
Category: time
Prefix:   time_*
```

**Syscalls**:

| Syscall | Args | Returns | Preconditions | Postconditions |
|---------|------|---------|----------------|----------------|
| `time_get_monotonic()` | none | nanoseconds (u64) | None | Returns monotonic clock |
| `time_get_wall()` | none | unix_timestamp_u64 | None | Returns wall clock |
| `time_sleep(nanoseconds)` | duration_ns | success | Duration >= 0 | Task woken after duration |
| `time_set_alarm(nanoseconds, target_endpoint)` | duration_ns, notify endpoint | alarm_id or error | Caller has CAP_TIME | Message sent to endpoint when expired |
| `time_cancel_alarm(alarm_id)` | alarm handle | success or error | Alarm exists | Alarm cancelled, no message sent |

**Guarantees**:
- Monotonic clock never goes backward
- Accuracy ±1ms (best effort)
- Alarms are delivered via IPC message

**Error Handling**:
- `EINVAL`: Invalid duration
- `ENOEXIST`: Alarm doesn't exist
- `EACCES`: No permission for alarm

---

## 5. IPC Transport Assumptions (Early Phase)

### 5.1 Message Format

All messages follow this structure:

```
[version: u16]
[msg_id: u32]
[sender_task_id: u32]
[reserved: u16]
[payload_size: u32]
[payload: variable]
```

- **version**: IPC protocol version (current: 1)
- **msg_id**: Sequence number, no implicit ordering
- **sender_task_id**: Always set by kernel, untrusted sender identity
- **payload**: User data, any format

### 5.2 Capabilities as Handles

- Capabilities are **opaque u64 handles**
- Kernel maps handle → capability internally
- Handles are **per-task local** (not globally unique)
- Passing invalid handle = EBADF

### 5.3 Early Phase Transport

**Initial**: Unix Domain Sockets (UDS)
- **Why**: Kernel-enforced access control, low latency
- **Later**: Shared memory rings with atomic operations
- **Not used**: Network sockets for internal IPC (too much overhead)

### 5.4 Backpressure & Flow Control

- Kernel endpoint has **fixed buffer size** (~4MB default)
- Sender gets `EAGAIN` if buffer full
- **Userspace manages backpressure** (retry, drop, or notify sender)
- No implicit blocking in kernel

---

## 6. Message Flow: CLI → Managementd → Kernel

```
┌──────────────────────────────────────────────────────────┐
│ User invokes: orbital package install example.pkg       │
└──────────────┬───────────────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────────────┐
│ CLI Process                                              │
│ - Parse command: "package install example.pkg"          │
│ - Validate local syntax                                 │
│ - Connect to managementd IPC endpoint                   │
│ - Send IPC message: {cmd: "package_install",            │
│                      args: "example.pkg"}                │
└──────────────┬───────────────────────────────────────────┘
               │
    IPC Transport (Unix Domain Socket)
               │
               ▼
┌──────────────────────────────────────────────────────────┐
│ Management Daemon (managementd)                          │
│ - Receive IPC from CLI                                  │
│ - Check RBAC: Does CLI user have package_install perm? │
│ - Fetch package file                                    │
│ - Validate package signature                            │
│ - Allocate task resources                               │
│ - Call kernel: task_create(installer_entry, ...)       │
│                                                         │
│ Sequence:                                               │
│   1. task_create() → task_id                            │
│   2. mem_alloc(pkg_size) → memory_addr                  │
│   3. Load package bytes at memory_addr                  │
│   4. task_wait(task_id, timeout=60s)                    │
│   5. Check exit code                                    │
└──────────────┬───────────────────────────────────────────┘
               │
    Kernel Syscalls (via sysenter/VMEXIT)
               │
               ▼
┌──────────────────────────────────────────────────────────┐
│ Kernel                                                   │
│ - Validate capability: task_create allowed?             │
│ - Allocate task structure                               │
│ - Set up stack, register state                          │
│ - Allocate virtual address space                        │
│ - Return task_id handle                                 │
│                                                         │
│ - Allocate memory pages for package                     │
│ - Map into task's address space                         │
│ - Return virtual address                                │
│                                                         │
│ - On task exit, set exit code                           │
│ - Return to managementd                                 │
└──────────────┬───────────────────────────────────────────┘
               │
    Syscall Return
               │
               ▼
┌──────────────────────────────────────────────────────────┐
│ Management Daemon (continued)                            │
│ - Check exit code                                       │
│ - Log result in audit log                               │
│ - Update service registry                               │
│ - Send response to CLI: {status: "ok", task_id: 42}    │
└──────────────┬───────────────────────────────────────────┘
               │
    IPC Response
               │
               ▼
┌──────────────────────────────────────────────────────────┐
│ CLI Process                                              │
│ - Receive response from managementd                     │
│ - Print: "Package installed as task 42"                │
│ - Exit                                                  │
└──────────────────────────────────────────────────────────┘
```

**Key Contract Points**:
1. CLI **never touches kernel** directly
2. CLI **never manipulates tasks** directly
3. All state changes go through **managementd only**
4. Kernel **enforces capabilities**, not policies
5. **Auditability**: Every decision is logged at managementd level

---

## 7. Failure Handling Rules

### 7.1 Kernel Failures (OOM, Exhaustion)

**What happens**:
- Syscall returns error code (e.g., `ENOMEM`)
- Kernel **does not panic**
- Caller must handle error

**Example**:
```
mem_alloc(1GB) → ENOMEM
```

**Userspace must**:
- Log error
- Notify managementd
- Trigger recovery (restart service, free memory, etc)

### 7.2 IPC Failures (Timeout, Dropped Message)

**What happens**:
- Sender gets `EAGAIN` (buffer full) or `ETIMEOUT` (no response)
- **No retry in kernel**
- Message is **not automatically retried**

**Userspace must**:
- Implement retry logic with backoff
- Notify user/service of failure
- Log for debugging

### 7.3 Task Crashes

**What happens**:
- Task runs until it calls `task_exit()` or crashes
- Kernel **does not automatically restart**
- Exit code available via `task_wait()`

**Userspace must** (supervisor):
- Detect exit
- Check exit code
- Implement restart policy
- Log incident

### 7.4 Capability Violations

**What happens**:
- Syscall returns `EACCES`
- **No panic, no core dump**
- Audit log entry created

**Userspace must**:
- Log security violation
- Alert administrator
- Possibly terminate offending task

### 7.5 Invalid Arguments

**What happens**:
- Syscall returns `EINVAL`
- Kernel checks arguments early
- Bad address → `EFAULT`

---

## 8. Versioning & Compatibility Rules

### 8.1 Syscall Versioning

**Guarantee**: Syscalls are **never removed**, only added.

**Old syscall → New kernel**: Works (backward compatible)
**New syscall → Old kernel**: Returns `ENOSYS`

**Userspace must**:
- Check `ENOSYS` gracefully
- Fall back to older behavior
- Not rely on new syscalls without checks

### 8.2 IPC Protocol Versioning

**Guarantee**: IPC protocol version is in every message.

**Old protocol → New kernel**: Kernel understands both
**New protocol → Old kernel**: Routing fails with version mismatch error

**Managementd must**:
- Check `ipc_get_version()` on each endpoint
- Use compatible protocol version
- Degrade gracefully if versions don't match

### 8.3 Capability Versioning

**Guarantee**: Capabilities carry their API version.

**Old capability → New kernel**: Works (forward compatible)
**New capability → Old kernel**: Returns `ENOTSUP`

### 8.4 Stability Guarantees

| Phase | Guarantee |
|-------|-----------|
| **Alpha** | No guarantees, breaking changes expected |
| **Beta** | Syscall ABI frozen, IPC proto frozen |
| **Stable** | Backward compatibility required for 10+ years |

**Current Phase**: Alpha (breaking changes are OK)

---

## 9. Capability Model

### 9.1 Capability Types

```
CAP_TASK_CREATE    - Create new tasks
CAP_TASK_MANAGE    - Suspend/resume tasks
CAP_TASK_KILL      - Forcibly terminate tasks
CAP_MEM_ALLOC      - Allocate memory
CAP_MEM_SHARE      - Share memory with other tasks
CAP_IPC            - Create IPC endpoints
CAP_TIME           - Set alarms
CAP_INTERRUPT      - Register interrupt handlers
CAP_LABEL          - Assign security labels
```

### 9.2 Capability Delegation

**Rules**:
- Only the owner can delegate a capability
- Receiver is identified by task_id
- Delegated capability **cannot be re-delegated** by default
- Kernel tracks delegation history (for audit)

**Example**:
```
Task A has CAP_MEM_SHARE
Task A calls: ipc_send_cap(task_B_endpoint, CAP_MEM_SHARE)
Task B receives capability in IPC message
Task B can now call: mem_share(...)
```

### 9.3 Capability Revocation

**Not implemented in Alpha phase.**

Future capability model will support:
- Time-bound capabilities
- Revocation by issuer
- Hierarchical delegation with limits

---

## 10. Observable Behavior Guarantees

### 10.1 Determinism

**Kernel guarantees**:
- Same inputs → same outputs (deterministic error codes)
- Scheduling is **not deterministic** (preemptive)
- IPC delivery order is **deterministic** (FIFO per endpoint)

### 10.2 Atomicity

**Syscalls are atomic** with respect to:
- Capability checks
- IPC message delivery
- Memory protection changes

**Not atomic across syscalls** (that's userspace coordination)

### 10.3 Audit Trail

**Kernel logs** (to audit service via IPC):
- All capability-related actions
- All syscalls (structured: timestamp, task_id, syscall, args, result)
- All IPC messages (sender, recipient, size, status)

**Userspace is responsible** for:
- Storing audit logs persistently
- Rotating logs
- Filtering sensitive data

---

## 11. Testing & Validation

### 11.1 Compatibility Testing

**Before any release**:
- [ ] Old binaries on new kernel: PASS
- [ ] New binaries on old kernel: Graceful ENOSYS
- [ ] IPC version mismatch: Detected and logged
- [ ] Capability version changes: Backward compatible

### 11.2 Stress Testing

**Before any release**:
- [ ] 10,000 tasks created/destroyed
- [ ] 1M IPC messages in 1 second
- [ ] 100GB memory allocation/free cycles
- [ ] No kernel panics (all errors are `EBADF`, `ENOMEM`, etc)

### 11.3 Security Testing

**Before any release**:
- [ ] Capability cannot be forged
- [ ] Capability cannot be used after revocation (future)
- [ ] Cross-task access is blocked
- [ ] IPC messages cannot be tampered with

---

## 12. Example: Safe Service Isolation

```
System State:
  Task A (package: firewall) - has CAP_INTERRUPT
  Task B (package: logger) - has CAP_IPC
  Task C (package: exploit) - no capabilities

Scenario: Exploit in Task C tries to intercept network traffic

Action:
  Task C calls: ipc_endpoint_create() // try to create endpoint
  Kernel checks: Does Task C have CAP_IPC?
  Result: NO → return EACCES

  Task C tries: task_create(...) // try to spawn task
  Kernel checks: Does Task C have CAP_TASK_CREATE?
  Result: NO → return EACCES

  Task C tries: mem_share(addr, task_B_id) // access logger's memory
  Kernel checks: Is addr in Task C's memory space?
  Result: NO → return ENOMAP

Final Result:
  Task C is ISOLATED
  Cannot compromise other services
  Cannot communicate with kernel directly
  All actions logged in audit trail
  Administrator can review and terminate Task C
```

---

## 13. Future Enhancements (Not This Phase)

- [ ] Capability revocation
- [ ] Shared memory rings (instead of UDS)
- [ ] Network syscalls (datagram IPC, routing hints)
- [ ] Real-time priority support
- [ ] Memory-mapped device access (device drivers)
- [ ] Signal delivery to tasks
- [ ] Process groups and job control
- [ ] Dynamic module loading

---

## Summary

| Aspect | Kernel | Userspace |
|--------|--------|-----------|
| **Memory** | Allocate, isolate | Quota enforcement |
| **Tasks** | Create, schedule | Restart policies |
| **IPC** | Deliver, access control | Routing, brokering |
| **Security** | Enforce capabilities | Implement policies |
| **Drivers** | Interrupt routing | Device-specific logic |
| **Decisions** | NONE | ALL state mutations |

**Golden Rule**: If it can be in userspace, it MUST be in userspace.

The kernel is a **microkernel by design**—tiny, fast, and correct.

---

**Document Version**: 0.1 (Alpha)  
**Last Updated**: January 16, 2026  
**Status**: Ready for Review

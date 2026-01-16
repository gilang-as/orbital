# IPC Transport Layer Design

**Status:** Implementation Complete  
**Date:** January 16, 2026  
**Version:** 1.0

## Overview

Orbital OS has a minimal IPC transport layer consisting of:

1. **Kernel:** Ring buffer primitive only (passes raw bytes)
2. **Common:** Message type definitions
3. **Userspace:** Protocol implementation, routing, and policy

This document explains what IS in the kernel, what ISN'T, and WHY.

## What IS in the Kernel

### 1. Ring Buffer Primitive (`kernel/src/ipc.rs`)

The kernel provides ONE abstraction: a lock-free ring buffer for passing raw bytes between processes.

```rust
pub struct RingBuffer {
    messages: UnsafeCell<Vec<RingMessage>>,
    write_index: AtomicUsize,    // Atomic for lock-free sync
    read_index: AtomicUsize,
}

pub struct RingMessage {
    pub sender_task_id: u32,     // Task ID (kernel-set)
    pub msg_id: u32,              // Application-defined message ID
    pub payload_len: u16,         // Payload size in bytes
    pub payload: [u8; 256],      // Raw bytes (userspace interprets)
}
```

**Key properties:**
- **Enqueue:** Copy message to ring, update atomic index (O(1))
- **Dequeue:** Read message from ring, update atomic index (O(1))
- **Lock-free:** No mutexes, only atomic operations
- **Fixed size:** 256 messages of 256 bytes each
- **Single writer, single reader:** Per endpoint (in real implementation with task isolation)

### 2. Type Definitions (`common/src/lib.rs`)

The common crate defines message types only:
- `RawIpcMessage`: The raw message type
- `IpcMessageHeader`: Protocol header metadata
- `MgmtCommand` / `MgmtResponse`: Application enums

**Critically: No implementation logic.** These are type definitions only.

### 3. Userspace Protocol (`userspace/ipc/src/lib.rs`)

Userspace wraps the kernel primitive and implements:
- **Serialization:** Command → bytes, bytes → response
- **Protocol versioning:** Version field checks
- **Error handling:** Retry logic, timeout policies
- **Client/Server interfaces:** IpcClient, IpcServer

## What is NOT in the Kernel (and Why)

### ❌ Message Serialization

**NOT in kernel.** Userspace handles all serialization/deserialization.

**Why:**
- Serialization formats vary by application (JSON, Protobuf, custom binary)
- Kernel can't predict all formats developers will use
- Serialization is deterministic logic that belongs in userspace
- Adds complexity and potential bugs to privileged code
- Kernel should handle only raw byte transport

**Example:** MgmtCommand serialization lives in `userspace/ipc/src/lib.rs`:
```rust
fn serialize_command(cmd: MgmtCommand) -> [u8; 4] {
    let mut bytes = [0u8; 4];
    bytes[0] = match cmd {
        MgmtCommand::GetState => 0,
        MgmtCommand::Shutdown => 1,
    };
    bytes
}
```

### ❌ Message Routing

**NOT in kernel.** Userspace IPC router distributes messages.

**Why:**
- Routing logic is policy (which process gets which message)
- Policies change per deployment (staging vs. production, different configurations)
- Policies are user-configurable
- Kernel providing routing means kernel changes for policy changes
- Routing requires maintaining recipient tables, which is mutable state
- If routing is in kernel, changing routing requires kernel updates

**Example policy:** "Only managementd can receive task_create requests"  
**Where it lives:** `userspace/managementd/` (userspace daemon)

**Alternative:** Could be in an IPC Router service in userspace.

### ❌ Access Control and Capabilities

**NOT in kernel.** Userspace enforces capability checks.

**Why:**
- Access control is policy (who can send to whom)
- Different systems have different security models (RBAC, capabilities, DAC, MAC)
- Kernel hardcoding one model locks users into that model
- Capabilities can be delegated/revoked at userspace level
- Ring buffer primitives have no concept of "permission to send"
- Kernel sees only raw bytes; doesn't understand message content or intent

**Example:** "Does this process have CAP_IPC_SEND?"  
**Where it lives:** `userspace/managementd/` (checks capabilities before accepting message)

### ❌ Message Framing and Headers

**NOT in kernel.** Userspace adds protocol headers.

**Why:**
- Framing (length prefixes, checksums, delimiters) is protocol-specific
- Different protocols have different framing requirements
- Kernel doesn't know if next 4 bytes are length, or metadata, or payload
- Framing is deterministic logic that belongs in userspace

**Example:** If a protocol needs 4-byte length prefix + checksum + payload:
```rust
// Userspace builds this structure
struct FramedMessage {
    len: u32,
    checksum: u32,
    payload: [u8; 248],
}
```

Kernel never sees this structure—just raw bytes.

### ❌ Backpressure and Send Failure Handling

**NOT in kernel.** Userspace handles retry logic.

**Why:**
- Ring buffer is fixed size (256 messages), can get full
- Kernel can't implement retry policy (what's the right retry count?)
- Kernel can't implement exponential backoff
- Different applications need different retry strategies
- Kernel returning "buffer full" is correct; application choosing what to do with that error belongs in userspace

**Example:** If `ring.enqueue()` returns `Err(())`:
```rust
// Userspace decides:
// - Retry N times with exponential backoff? ✓
// - Drop the message? ✓
// - Block the sender? ✓
// - Log and propagate error to caller? ✓
// Kernel can't decide—it doesn't know the application semantics
```

### ❌ Receive Timeouts and Blocking Policies

**NOT in kernel.** Userspace implements wait strategies.

**Why:**
- "Wait for message" semantics vary: block forever? timeout after 1s? poll?
- Different applications need different blocking semantics
- Kernel can't implement all timeout strategies efficiently
- Kernel providing only `dequeue()` → `Option<Message>` is correct
- Userspace builds blocking/polling on top

**Example userspace wrapper:**
```rust
pub fn receive_with_timeout(&self, duration: Duration) -> Result<Message, TimeoutError> {
    let start = Instant::now();
    loop {
        if let Some(msg) = self.ring.dequeue() {
            return Ok(msg);
        }
        if start.elapsed() > duration {
            return Err(TimeoutError);
        }
        // Sleep briefly and retry (userspace decides sleep duration)
        std::thread::sleep(Duration::from_millis(1));
    }
}
```

### ❌ Message Ordering Guarantees Across Endpoints

**NOT in kernel.** This is policy/coordination that lives in userspace.

**Why:**
- "Message A sent before B must be received before B" requires synchronization
- This is only meaningful for specific application semantics
- Kernel can't guarantee ordering across independent endpoints
- If needed, application adds sequence numbers (userspace protocol detail)

**Example:** managementd might care about ordering, adds sequence numbers:
```rust
struct ProtocolMessage {
    sequence_number: u64,  // Userspace adds this
    command: MgmtCommand,
}
```

### ❌ Per-Sender Quotas and Rate Limiting

**NOT in kernel.** Userspace enforces resource policies.

**Why:**
- "This process can send max 1000 messages/sec" is policy
- Quota enforcement is deterministic logic
- Different processes have different quota requirements
- Kernel hardcoding quotas requires kernel changes when policies change
- Userspace IPC router can track sender stats and enforce limits

### ❌ Configuration Parsing

**NOT in kernel.** Userspace reads all configuration.

**Why:**
- Configuration is runtime state (changeable without recompile)
- Kernel must be recompiled to change config
- Config parsing is userspace responsibility (where policies live)
- Kernel takes only basic parameters needed to allocate memory

**Example:** No TOML, JSON, or config file parsing in kernel—that's in userspace daemon.

### ❌ Driver Integration and Hardware Specific Logic

**NOT in kernel.** Userspace handles hardware integration.

**Why:**
- While kernel might provide primitives for shared memory, the actual mapping to hardware (like IOMMU, DMA rings) is userspace-orchestrated
- Different hardware has different requirements
- Driver bugs should not crash the kernel

### ❌ Business Logic

**NOT in kernel.** Only move bytes.

**Why:**
- Kernel provides transport; applications implement logic
- Kernel has no context for what messages mean
- If kernel knows about "shutdown" logic, "package deployment", "configuration updates"—kernel is too smart
- Kernel should have single responsibility: move bytes safely

## Design Principles Applied

### 1. Kernel Minimalism

> The kernel's job is to provide **primitives**, not **policies**.

Ring buffer is a primitive (move bytes safely). Routing, authentication, serialization, retry logic—all policies.

### 2. Separation of Concerns

| Concern | Owner | Reason |
|---------|-------|--------|
| Byte transport | Kernel | Low-level, platform-dependent |
| Message format | Userspace | Protocol-specific |
| Routing | Userspace | Policy-specific |
| Access control | Userspace | Deployable policy |
| Error handling | Userspace | Application-specific strategy |

### 3. Failures in Userspace Are Safe

If userspace crashes, kernel is unaffected. Userspace can be restarted.  
If kernel has complex logic and crashes, system fails.

### 4. Policies Remain Flexible

Ring buffer is fixed, primitives unchanged. Everything else (routing rules, serialization formats, rate limits) is configurable in userspace without touching kernel.

## Current Implementation Status

✅ **Complete:**
- `kernel/src/ipc.rs`: Ring buffer (243 lines)
- `common/src/lib.rs`: Message types (RawIpcMessage, MgmtCommand, MgmtResponse)
- `userspace/ipc/src/lib.rs`: IpcClient/IpcServer wrappers with serialization stubs

✅ **Tested:**
- Ring buffer enqueue/dequeue tests pass
- IpcClient serialization tests pass
- IpcServer instantiation tests pass

🚧 **Next Phase (Future):**
1. Integrate ring buffer into syscall interface
2. Map ring buffer to task memory
3. Implement actual userspace daemon reading/writing ring buffer
4. Add capability checking in managementd
5. Implement IPC Router for distributing messages

## Future Enhancements (Out of Scope - Userspace)

### Shared Memory Rings (Alternative to Fixed Ring Buffer)

Currently: Single 256-message ring buffer.

Future: Could upgrade to shared memory ring implementation:
- Allows variable-sized messages
- Allows applications to allocate their own IPC buffers
- Still no kernel policy—just primitives

Kernel change: Provide syscall to create shared memory region.  
Userspace: Implement ring buffer protocol on top.

### Capability Delegation Chains

"Process A has CAP_IPC to talk to B; can B delegate CAP_IPC to C?"

Kernel: No change (kernel doesn't check capabilities).  
Userspace: Implementable through managementd policy.

### Message Ordering

If applications need ordered delivery:

Kernel: No change (still just rings).  
Userspace: Add sequence numbers to messages, reorder in userspace if needed.

### Backpressure Flow Control

If sender is filling ring buffer too fast:

Kernel: No change.  
Userspace: Sender sleeps/retries when ring full; receiver can apply back pressure through response messages.

## Conclusion

The IPC layer is deliberately minimal: the kernel provides lock-free message passing primitives, and userspace owns all policy, serialization, routing, and error handling. This design allows:

- **Kernel safety:** Minimal attack surface, fewer bugs
- **Policy flexibility:** Change rules without recompiling kernel
- **System resilience:** Userspace failures don't crash kernel
- **Development agility:** Userspace IPC improvements don't require kernel changes

The separation is enforced by design: the ring buffer interface exposes no policy mechanisms—just raw byte passing with atomic indices.

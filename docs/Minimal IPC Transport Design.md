# Minimal IPC Transport Design

**Status:** Design Contract  
**Date:** January 16, 2026  
**Version:** 1.0  
**Scope:** IPC transport primitives only (no protocol, no policy, no feature logic)

---

## 1. Purpose of IPC in Orbital OS

The IPC (Inter-Process Communication) system enables userspace processes to send and receive messages. It is the foundation for:

- **Management daemon communication:** Processes requesting state changes
- **Task lifecycle events:** Notifications between services
- **Data plane operations:** Processes coordinating work

**Critical constraint:** IPC is a **transport mechanism only**. It does not enforce policy, routing decisions, or business logic.

---

## 2. Design Goals

### Primary Goals

1. **Minimal kernel code:** Kernel provides only byte-passing primitives, nothing else
2. **Deterministic behavior:** Messages are moved without blocking, waiting, or interpretation
3. **Safe isolation:** No process can corrupt another's message or bypass memory protection
4. **Policy-free kernel:** Kernel makes no decisions about who can send, receive, or what messages mean

### Secondary Goals

1. **Bounded latency:** IPC operations complete in bounded time (no unbounded loops in kernel)
2. **Observable operations:** IPC behavior is predictable and testable
3. **Userspace flexibility:** Policies, retries, and protocol are entirely userspace

---

## 3. Kernel IPC Primitives

### What Kernel Provides

The kernel provides **one abstraction**: a message queue primitive.

#### 3.1 Message Structure

```
Message {
  sender_task_id: u32          // Set by kernel (task ID of sender)
  msg_id: u32                  // Application-defined (userspace sets)
  payload_len: u16             // Number of valid bytes in payload
  payload: [u8; MAX_SIZE]      // Raw bytes (kernel never interprets)
}
```

**Constraints:**
- `MAX_SIZE` = 256 bytes (fixed, no variable-length messages in kernel)
- `sender_task_id` is set by kernel, not by application
- `payload` is opaque to kernel (kernel copies bytes, never reads content)
- `msg_id` is reserved for application use (kernel ignores)

#### 3.2 Queue Operations

**`enqueue(message) → Result`**
- **Input:** Message struct
- **Output:** `Ok(())` on success, `Err(())` if queue full
- **Semantics:** Copy message to queue atomically
- **Blocking:** Non-blocking (returns immediately)
- **Guarantees:** Atomic copy; sender cannot see partial writes
- **Failure mode:** If queue is full, reject with error (no waiting, no overflow)

**`dequeue() → Option<Message>`**
- **Input:** None
- **Output:** `Some(message)` if message available, `None` if queue empty
- **Semantics:** Read message from queue atomically
- **Blocking:** Non-blocking (returns immediately)
- **Guarantees:** Atomic read; receiver gets message in order (FIFO)
- **Failure mode:** If queue empty, return `None` immediately

#### 3.3 Queue Properties

- **Capacity:** Fixed size (kernel choice; recommended 256 messages)
- **Ordering:** FIFO (First-In-First-Out)
- **Atomicity:** Enqueue and dequeue are atomic operations
- **Concurrency:** Lock-free; uses atomic indices (no mutexes)
- **Fairness:** No priority; all messages queued equally

### What Kernel Does NOT Provide

The kernel does **not** provide:

- ❌ Message routing (which process receives this message)
- ❌ Access control (can process A send to process B)
- ❌ Serialization (converting data types to/from bytes)
- ❌ Blocking/waiting (receive with timeout)
- ❌ Message filtering (receive only specific message types)
- ❌ Backpressure handling (retry on queue full)
- ❌ Protocol versioning (compatibility checks)
- ❌ Configuration parsing (queue sizes, policies)
- ❌ Message validation (size checks, checksum validation)
- ❌ Message ordering guarantees across multiple queues

---

## 4. Userspace Responsibilities

### 4.1 Protocol Definition

Userspace defines:
- Message format (what bytes mean what)
- Serialization rules (command ↔ bytes)
- Protocol versioning and compatibility
- Message types and enums

### 4.2 Routing and Delivery

Userspace implements:
- **IPC Router:** Determines which process receives which message
- **Process discovery:** How senders find recipient process IDs
- **Endpoint management:** Creating/destroying message queues per process

### 4.3 Access Control and Policy

Userspace enforces:
- **Capability checks:** Can process A send to B?
- **Rate limiting:** How many messages per second per sender?
- **Message validation:** Is this message well-formed?
- **Resource quotas:** How much queue space per process?

### 4.4 Error Handling and Retry

Userspace handles:
- **Send failures:** Queue full → retry with backoff
- **Receive timeouts:** No message → wait or error?
- **Protocol errors:** Invalid message format → recover?
- **Deadlock prevention:** Avoiding cyclic message dependencies

### 4.5 Blocking and Synchronization

Userspace implements:
- **Blocking receive:** Sleep until message arrives
- **Non-blocking receive:** Check queue, return immediately
- **Receive timeout:** Wait up to N milliseconds
- **Polling strategies:** Efficient waiting for messages

### 4.6 Message Ordering Guarantees

If application requires ordered delivery across multiple messages:
- Userspace adds sequence numbers
- Userspace tracks delivery order
- Userspace reorders if needed

### 4.7 Bidirectional Communication

For request-response patterns:
- Userspace tracks which messages are responses to which requests
- Userspace matches responses to requests
- Userspace handles missing responses (timeout/retry)

---

## 5. Message Flow Lifecycle

### 5.1 Send Path (Process A → Queue)

1. **Process A creates message** with:
   - Command/data (serialized to bytes by userspace)
   - Destination process ID (determined by routing in userspace)
   - Message ID (tracking ID assigned by userspace)

2. **Process A calls kernel** `enqueue(message)`:
   - Kernel verifies message size ≤ MAX_SIZE
   - Kernel reads `sender_task_id` from task context
   - Kernel atomically copies message to destination queue
   - Kernel returns `Ok(())` or `Err(())` immediately

3. **If `Ok(())`:** Message queued successfully
   - Userspace continues (no blocking)
   - Userspace may wait for response (separate message)

4. **If `Err(())`:** Queue was full
   - Userspace decides: retry, backoff, fail, or drop

**Key properties:**
- Send is non-blocking
- Queue full is explicit error (not silent overflow)
- Kernel doesn't know what message contains
- Kernel doesn't route the message

### 5.2 Receive Path (Queue → Process B)

1. **Process B calls kernel** `dequeue()`:
   - Kernel atomically reads next message from queue (if available)
   - Kernel returns `Some(message)` or `None` immediately

2. **If `Some(message)`:** Message available
   - Process B receives message in hand
   - Message contains `sender_task_id` (set by kernel)
   - Process B deserializes payload (userspace protocol)

3. **If `None`:** Queue empty
   - Process B gets immediate return (non-blocking)
   - Process B decides: retry, sleep, timeout, or return error

**Key properties:**
- Receive is non-blocking
- No waiting in kernel
- Message order is FIFO
- Userspace implements wait semantics

### 5.3 Complete Request-Response Cycle

```
[Process A]                      [Kernel]           [Process B]
    │                               │                   │
    │─ serialize command ──→        │                   │
    │                               │                   │
    │─ enqueue(msg) ────────────→   │→ copy to queue    │
    │←────────────── Ok/Err ────────│                   │
    │                               │   ↓dequeue()      │
    │                               │←─────────────┐    │
    │                               │           [receive]
    │                               │              │
    │                               │          [process]
    │                               │              │
    │                               │   enqueue(response)
    │                               │←──────────→  │
    │                               │   [copy]     │
    │                               │              │
    │    ↓dequeue()                 │              │
    │←──────────────────────────┐   │              │
    │  [receive response]       │   │              │
    │  [deserialize]            │   │              │
    │  [done]                   │   │              │
    │                           │   │              │
```

---

## 6. Blocking vs Non-Blocking Behavior

### 6.1 Kernel Behavior (Non-Blocking)

Kernel provides **only non-blocking operations:**

| Operation | Blocking | Returns |
|-----------|----------|---------|
| `enqueue()` | Non-blocking | `Ok(())` or `Err(())` immediately |
| `dequeue()` | Non-blocking | `Some(msg)` or `None` immediately |

The kernel **never waits** for:
- Queue space (returns error instead)
- Message arrival (returns None instead)
- Response messages (not kernel's concern)

### 6.2 Userspace Blocking Strategies

Userspace builds blocking on top:

**Strategy 1: Sleep-Poll Loop**
```
while queue_empty() {
    sleep(1ms)
    check_timeout()
}
message = dequeue()
```

**Strategy 2: Event-Driven (Future)**
```
register_on_message_available()
wait_for_notification()
message = dequeue()
```

**Strategy 3: Thread-Per-Queue (Future)**
```
thread {
    loop {
        msg = dequeue()
        if msg { handle(msg) }
        else { sleep(1ms) }
    }
}
```

### 6.3 Blocking Guarantee

Kernel guarantees: **Enqueue and dequeue never block.**

This enables userspace to implement any waiting strategy without kernel overhead.

---

## 7. Error Handling and Backpressure

### 7.1 Queue Full (Send Failure)

**Situation:** Process A tries to send, but queue is full.

**Kernel behavior:** Return `Err(())` immediately.

**Userspace options:**
- **Retry:** Sleep and try again (with exponential backoff)
- **Fail:** Return error to caller
- **Drop:** Discard message silently
- **Buffer:** Store locally and retry later
- **Notify sender:** Send message to sender process (for flow control)

Kernel makes **no decision** about policy. Userspace chooses strategy per application.

### 7.2 Queue Empty (Receive Failure)

**Situation:** Process B tries to receive, but queue is empty.

**Kernel behavior:** Return `None` immediately.

**Userspace options:**
- **Retry:** Loop with sleep
- **Timeout:** Loop with timeout
- **Async:** Wait for notification (future)
- **Error:** Return error to caller

### 7.3 Malformed Messages

If userspace deserializes a message and it's invalid:

**Kernel behavior:** Kernel never validates (delivers raw bytes).

**Userspace behavior:**
- Log error
- Discard message
- Send error response to sender
- Increment error counter

Kernel provides the message; userspace decides if it's valid.

### 7.4 Message Loss

**Situation:** Queue becomes full and message is dropped.

**Guarantees:**
- Kernel guarantees: Enqueued messages are delivered (no kernel loss)
- Not guaranteed: Application-level delivery (if queue full, app drops it)

**Userspace responsibility:**
- If messages must not be lost, app must not allow queue to fill
- Userspace enforces quotas to prevent overrun
- Userspace implements retry-on-rejection

### 7.5 Backpressure

**Definition:** Sender detects receiver is slow (queue backing up).

**Kernel support:** Return `Err(())` when queue full (explicit backpressure signal).

**Userspace handling:**
- Monitor send failures
- Throttle sender (reduce message rate)
- Notify sender of queue state
- Implement flow control protocol

---

## 8. Non-Goals

This design explicitly does **not** address:

### ❌ Kernel-Enforced Policies

The kernel will not:
- Decide which process can send to which
- Enforce message rate limits
- Validate message format
- Implement routing rules
- Check message checksums
- Parse configuration files
- Implement authentication/authorization

### ❌ Advanced Message Features

The kernel will not provide:
- Variable-length messages (sized exactly at 256 bytes)
- Message priorities (FIFO only)
- Conditional receive (can't filter by sender/type in kernel)
- Message expiration (no TTL)
- Reliable delivery guarantees (queue can overflow)
- End-to-end encryption (kernel moves plaintext)
- Compression (kernel copies bytes as-is)

### ❌ Performance Optimizations

The kernel will not optimize for:
- Very large message workloads (256 bytes max is the limit)
- Inter-CPU message passing (single kernel manages all queues)
- Cache-optimized buffer layouts (userspace can implement that)
- Lock-free algorithms beyond atomic indices (sufficient for single-reader/single-writer)

### ❌ Advanced Concurrency

The kernel will not handle:
- Multiple readers on same queue (serialize reader access in userspace)
- Multiple writers on same queue (serialize writer access in userspace)
- Fair scheduling of message delivery (userspace implements fairness)
- Interrupt-driven delivery (userspace polls)

### ❌ Persistence

The kernel will not:
- Persist messages to disk
- Replay messages after reboot
- Provide message history
- Log all messages

All persistence is userspace responsibility.

---

## 9. Design Principles Summary

| Principle | Application |
|-----------|-------------|
| **Kernel minimalism** | Kernel does byte transport only; everything else is userspace |
| **No kernel policy** | Kernel never checks permissions, routes, or validates |
| **Non-blocking primitives** | Kernel never waits; userspace implements wait strategies |
| **Explicit errors** | Kernel returns errors immediately; userspace decides policy |
| **Safe isolation** | Messages are isolated; no process can corrupt another |
| **Observable behavior** | All IPC operations are deterministic and testable |

---

## 10. Contract Verification Checklist

When implementing IPC, verify:

- ✅ Kernel `enqueue()` copies message atomically and returns immediately
- ✅ Kernel `dequeue()` reads message atomically and returns immediately
- ✅ Kernel never interprets message payload
- ✅ Kernel sets `sender_task_id` from task context
- ✅ Queue full returns `Err(())`, doesn't wait or overflow
- ✅ Queue empty returns `None`, doesn't wait
- ✅ Routing is implemented in userspace, not kernel
- ✅ Serialization is userspace responsibility
- ✅ Access control is userspace responsibility
- ✅ Retries are userspace responsibility
- ✅ Blocking/waiting is userspace responsibility
- ✅ No configuration parsing in kernel
- ✅ No message validation in kernel
- ✅ No policy enforcement in kernel

---

## 11. Future Work (Out of Scope)

Potential enhancements that remain userspace:

- **Shared memory rings:** Userspace implements variable-size message transport on shared memory primitives
- **Event notifications:** Kernel signals when queue has messages (userspace still polls for efficiency)
- **Deadline support:** Kernel doesn't care; userspace tracks deadlines
- **Message groups:** Userspace groups related messages
- **Ordering across queues:** Userspace implements if needed (sequence numbers)
- **Fair scheduling:** Userspace implements if needed
- **Encryption:** Userspace layer above IPC
- **Compression:** Userspace layer above IPC

The contract remains: kernel provides only the primitive, userspace owns all features.

---

## Conclusion

The IPC transport is a **strict contract**:

- **Kernel:** Atomically move bytes between processes (non-blocking)
- **Userspace:** Everything else (protocol, routing, policy, blocking, validation)

This separation ensures:
- Kernel remains minimal and verifiable
- Policies are changeable without kernel modifications
- Failures in userspace don't crash kernel
- Different systems can implement different policies with same kernel

The contract is enforced by design: the queue interface exposes no policy mechanisms—only raw byte passing with atomic indices.

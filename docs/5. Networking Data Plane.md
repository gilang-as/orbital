# Orbital OS – Networking Data Plane

> **Purpose**
>
> This document defines the **networking data plane architecture** of Orbital OS. The data plane is responsible for high-performance packet processing, forwarding, and enforcement, while remaining deterministic, observable, and resilient.

---

## 1. Design Goals

* High throughput and low latency
* Deterministic behavior under load
* Clear separation from control plane
* Zero-copy where possible
* Multi-core scalability
* Hardware offload awareness

---

## 2. Plane Separation

### 2.1 Data Plane Responsibilities

* Packet RX/TX
* L2/L3 forwarding
* NAT execution
* Firewall enforcement
* QoS classification and scheduling

### 2.2 Control Plane Responsibilities (Out of Scope)

* Routing protocols
* Configuration parsing
* Policy definition
* User management

The data plane **never parses configuration** directly.

---

## 3. Packet Processing Model

### 3.1 Processing Pipeline

```
NIC RX
  ↓
RX Queue
  ↓
Packet Buffer
  ↓
Fast-Path Classification
  ↓
[Optional Slow-Path Hook]
  ↓
TX Queue
  ↓
NIC TX
```

---

### 3.2 Fast Path vs Slow Path

**Fast Path**:

* Stateless forwarding
* Simple NAT
* Firewall allow/deny
* QoS tagging

**Slow Path**:

* Complex inspection
* Control-plane interaction
* Exceptional packets

Slow path must be rate-limited.

---

## 4. Packet Buffers

### 4.1 Buffer Design

* Fixed-size buffers
* Cache-aligned
* Reusable via pools

### 4.2 Zero-Copy Strategy

* Avoid data copies between RX/TX
* Share buffers across pipeline stages
* Copy only when absolutely required

---

## 5. Concurrency Model

* Per-CPU packet processing
* Lock-free queues
* Batch processing

Packets never migrate between CPUs by default.

---

## 6. NIC Driver Integration

### 6.1 Driver Model

* Polling-based (NAPI-like)
* Multi-queue support
* Interrupt moderation

### 6.2 RSS and Affinity

* RSS distributes flows
* CPU affinity preserved per flow

---

## 7. Forwarding Engine

### 7.1 Lookup Structures

* Routing tables
* Neighbor tables
* NAT tables

Requirements:

* Lock-free reads
* Versioned updates

---

### 7.2 Table Updates

* Prepared by control plane
* Atomically swapped
* Never modified in place

---

## 8. Firewall and NAT

### 8.1 Firewall Hooks

* Pre-routing
* Post-routing
* Local input/output

Rules are precompiled for fast-path use.

---

### 8.2 NAT Processing

* Stateless where possible
* Stateful with bounded resources
* Connection tracking optimized

---

## 9. QoS and Traffic Control

* Classification on ingress
* Priority queues
* Rate limiting

QoS decisions must be lightweight.

---

## 10. Hardware Offload Awareness

* Detect NIC capabilities
* Offload checksum, segmentation, filtering
* Fallback to software processing

Offloads must be transparent to control plane.

---

## 11. Userspace Interaction

### 11.1 Data Plane API

Exposed to control plane:

* Table updates
* Policy reload
* Statistics queries

No per-packet userspace calls.

---

### 11.2 Telemetry

* Packet counters
* Drop reasons
* Queue statistics

Telemetry is read-only.

---

## 12. Failure Handling

* Packet drops preferred over stalls
* Resource exhaustion protection
* Graceful degradation under load

Data plane never blocks kernel scheduling.

---

## 13. Security Considerations

* Bounds-checked parsing
* No untrusted pointer usage
* Strict validation on slow path

---

## 14. Performance Strategy

* Optimize for common case
* Avoid branch-heavy logic
* Prefer static dispatch

Performance regressions are treated as bugs.

---

## 15. Development Guidelines

* Rust with minimal `unsafe`
* Extensive benchmarking
* Deterministic test cases

---

## 16. Next Documents

* Orbital OS – Management Plane (CLI & API)
* Orbital OS – Update & Recovery System

---

**End of Document**

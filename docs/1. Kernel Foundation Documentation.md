# Orbital OS – Kernel Foundation Documentation

> **Purpose**
>
> This document defines the **kernel foundation design** for Orbital OS — a lightweight, resilient, modular, and open-source **network operating system**, built from scratch using **Rust**.
>
> This document intentionally focuses on **kernel and system foundations**, not end-user features (hotspot, VPN, etc.), ensuring that future features can be developed safely, stably, and at scale.

---

## 1. Design Goals

### 1.1 Primary Goals

* Lightweight and deterministic behavior
* High network throughput
* Fault-tolerant (no kernel panic due to service or user error)
* Modular and package-based architecture
* Secure by default
* Update-friendly (A/B partitions, rollback support)

### 1.2 Non-Goals

* Desktop or GUI operating system
* Full POSIX compatibility
* Linux userspace compatibility

---

## 2. Kernel Philosophy

### 2.1 Kernel Type

**Hybrid Kernel** with the following principles:

* Minimal kernel surface area
* Networking fast-path close to the kernel
* Complex logic implemented in user-space services

The kernel is responsible only for:

* Memory management
* Scheduler
* Inter-process communication (IPC)
* Networking fast path
* Driver and hardware abstraction
* Core security primitives

---

## 3. Kernel Responsibilities

### 3.1 Memory Management

* Virtual memory management
* Page allocator
* Slab allocator for kernel objects
* Zero-copy buffer support for networking

Requirements:

* No memory leaks
* Deterministic allocation failure handling
* Strong memory isolation between processes

---

### 3.2 Scheduler

* Preemptive scheduler
* SMP-aware
* CPU affinity support
* Real-time priorities for data plane workloads

Process types:

* Kernel threads
* System services
* User services (packages)

---

### 3.3 Process and Thread Model

* Mandatory process isolation
* Lightweight threading model (no fork-heavy semantics)
* Capability-based access control

Each **package runs as one or more isolated processes**.

---

## 4. Inter-Process Communication (IPC)

### 4.1 IPC Goals

* High performance
* Strong safety guarantees
* Observable and debuggable
* Versioned interfaces

### 4.2 IPC Mechanisms

* Message passing
* Shared memory (ring buffers)
* Event-based notifications

No global shared mutable state is allowed.

---

## 5. Networking Architecture

### 5.1 Plane Separation

#### Data Plane

* Packet receive and transmit
* Forwarding
* NAT
* Firewall hooks
* QoS hooks

#### Control Plane

* Routing daemons
* Interface configuration
* Policy management

The kernel handles **data plane fast-path only**.

---

### 5.2 Packet Processing Pipeline

1. NIC RX
2. Kernel packet buffer
3. Fast-path decision
4. Optional userspace hook
5. NIC TX

Design principles:

* Zero-copy where possible
* Lock-free data structures
* Batch-based processing

---

## 6. Driver and Hardware Abstraction

### 6.1 NIC Driver Model

* Polling-based (NAPI-like)
* Multi-queue support
* RSS-aware
* Hardware offload detection

### 6.2 Hardware Abstraction Layer (HAL)

* CPU
* Timers
* Interrupts
* Network interfaces
* Storage devices

---

## 7. Filesystem and Storage

### 7.1 Filesystem Goals

* Read-only root filesystem
* Writable configuration and state partitions
* Atomic write guarantees
* Power-loss safe design

### 7.2 Storage Layout (Conceptual)

* Boot partition
* Kernel A / Kernel B
* RootFS A / RootFS B
* Configuration partition
* Log partition

---

## 8. Configuration Foundation

> **The configuration system is a core value of Orbital OS**

The kernel provides:

* Atomic file operations
* Locking primitives

User-space services are responsible for:

* Candidate configuration
* Validation
* Commit and rollback logic

---

## 9. Logging and Observability

### 9.1 Kernel Logging

* Structured logging
* Ring-buffer based
* Rate-limited

### 9.2 Crash Handling

* Service crashes never cause kernel panic
* Crash dump support
* Diagnostic boot mode

---

## 10. Security Model

### 10.1 Capability-Based Security

* No global root user
* Per-process capability assignment

Examples:

* net_admin
* iface_control
* route_modify

---

### 10.2 User and Permission Foundation

The kernel provides:

* UID/GID abstraction
* Permission hooks

Policy enforcement is handled in user-space.

---

## 11. Update System Support

### 11.1 OS Updates

* A/B partition model
* Signed update images
* Automatic rollback on failure

### 11.2 Package Updates

* Independent package lifecycle
* Versioned IPC contracts
* Dependency-aware updates

The kernel guarantees:

* Safe service restarts
* Proper resource cleanup

---

## 12. Fault Tolerance Principles

* No single service can crash the operating system
* Kernel errors must be contained
* Watchdog support is mandatory
* Health-check hooks for system services

---

## 13. Development Constraints

* Rust (`no_std` for kernel code)
* Minimal and auditable `unsafe` usage
* Explicit error handling (panic-free design)

---

## 14. References

* [https://github.com/phil-opp/blog_os](https://github.com/phil-opp/blog_os)
* Linux networking stack design
* FreeBSD netgraph
* JunOS architecture
* OpenWRT and procd

---

## 15. Next Documents

* Orbital OS – Userspace Architecture
* Orbital OS – Package System
* Orbital OS – Networking Data Plane
* Orbital OS – Configuration System

---

**End of Document**
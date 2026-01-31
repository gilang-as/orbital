# Orbital OS – IPC & API Design

> **Purpose**
>
> This document defines the Inter-Process Communication (IPC) and API architecture of Orbital OS. The IPC/API layer is the backbone of the control plane, enabling fast, safe, and structured communication between CLI, management services, packages, and the kernel-facing components.

---

## 1. Design Goals

* Extremely low latency
* Strong typing and schema enforcement
* Clear versioning and compatibility rules
* Separation between control plane and data plane
* No direct kernel manipulation from user-facing tools

---

## 2. Architectural Principles

* **Single Source of Truth**: All state mutations flow through the management daemon
* **Fast Path vs Slow Path** separation
* **Explicit contracts** between components
* **Fail-safe behavior** over fail-fast

---

## 3. IPC Overview

```
CLI / Packages
      ↓
IPC API (Unix Domain Socket)
      ↓
Management Daemon
      ↓
Control Plane Services
      ↓
Kernel / Data Plane
```

---

## 4. IPC Transport

### 4.1 Unix Domain Sockets (UDS)

Primary transport for internal communication.

Reasons:

* No TCP/IP stack overhead
* Kernel-level permission enforcement
* Very low latency

---

### 4.2 Protocol Choice

Recommended:

* **gRPC over UDS** (using Protobuf)

Alternatives:

* Custom binary protocol
* Cap’n Proto (optional)

---

## 5. API Layers

### 5.1 Internal API (Fast Path)

Used by:

* CLI
* Core packages
* Internal services

Characteristics:

* Binary protocol
* Persistent connections
* Strongly typed

---

### 5.2 External API (Slow Path)

Used by:

* Web UI
* Automation tools
* External systems

Characteristics:

* HTTP-based
* JSON payloads
* Optional and disable-able

---

## 6. API Versioning

* Semantic versioning (v1, v2, ...)
* Explicit compatibility guarantees
* Backward compatibility preferred

Breaking changes require new major versions.

---

## 7. Service Boundaries

Each subsystem exposes its own API surface:

* Interface service
* Routing service
* Firewall service
* User & RBAC service
* Package service
* Update service

Services communicate only via IPC APIs.

---

## 8. Error Model

* Structured error codes
* Human-readable messages
* Machine-parseable metadata

Errors never crash the system.

---

## 9. Security Model

* IPC socket permission-based access
* Capability checks per request
* Mandatory authentication context

No anonymous IPC access.

---

## 10. Performance Considerations

* Zero-copy where possible
* Batching operations
* Streaming APIs for bulk updates

---

## 11. Observability

* Per-call latency metrics
* Error rate tracking
* Trace IDs across services

---

## 12. Development Guidelines

* Protobuf definitions are the source of truth
* CLI and REST share the same backend
* No business logic duplication

---

## 13. Failure Scenarios

Handled cases:

* Management daemon restart
* Partial service failure
* IPC socket disruption

System remains recoverable.

---

## 14. Testing Strategy

* Contract tests for APIs
* Fuzzing IPC inputs
* Fault injection

---

## 15. Next Document

* Orbital OS – Configuration System

---

**End of Document**

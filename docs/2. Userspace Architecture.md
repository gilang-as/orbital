# Orbital OS – Userspace Architecture

> **Purpose**
>
> This document defines the **userspace architecture** of Orbital OS. It describes how system services, feature packages, and management components run **outside the kernel**, communicate safely, and remain fault-tolerant.
>
> The userspace design ensures that feature failures never compromise kernel stability, while enabling modular development and independent updates.

---

## 1. Userspace Design Principles

* Strong isolation between services
* Crash containment (fail-fast, restart-safe)
* Clear separation of responsibilities
* Explicit contracts between components
* Minimal kernel dependencies

Userspace is where **most system logic lives**.

---

## 2. Userspace Layer Overview

```
+------------------------------+
| Management Plane             |
| CLI / API / Auth / RBAC      |
+------------------------------+
| Control Plane Services       |
| Routing / Config / Policy    |
+------------------------------+
| Feature Packages             |
| VPN / Hotspot / Firewall     |
+------------------------------+
| Core System Services         |
| Init / Supervisor / IPC      |
+------------------------------+
| Kernel                       |
+------------------------------+
```

---

## 3. Core System Services

### 3.1 Init Service

Responsibilities:

* First userspace process
* Spawns all core services
* Transitions system to operational state

Properties:

* Minimal logic
* Never restarted automatically

---

### 3.2 Service Supervisor

Responsibilities:

* Start, stop, restart services
* Monitor health checks
* Enforce restart policies
* Isolate crashes

Service states:

* Starting
* Running
* Failed
* Restarting
* Disabled

---

### 3.3 IPC Router / Broker

Responsibilities:

* Message routing between services
* IPC version negotiation
* Access control enforcement

Design:

* No service-to-service direct trust
* All IPC contracts are explicit

---

## 4. Service Model

### 4.1 Service Characteristics

Each service:

* Runs as a separate process
* Has its own capability set
* Has defined IPC interfaces
* Can be restarted independently

Services **must be stateless by default**.

---

### 4.2 Capability Model

Examples:

* `net.forward`
* `net.configure`
* `iface.manage`
* `user.manage`

Capabilities are:

* Assigned at service start
* Enforced by kernel hooks

---

## 5. Configuration Service

> **Single source of truth for system configuration**

Responsibilities:

* Maintain candidate configuration
* Validate configuration changes
* Commit or rollback atomically

Other services:

* Subscribe to config changes
* Never modify config directly

---

## 6. Management Plane

### 6.1 CLI Service

Responsibilities:

* Interactive command interface
* Human-friendly error messages
* Role-based access enforcement

CLI does not apply config directly.

---

### 6.2 API Service

Responsibilities:

* REST or gRPC interface
* Automation and integration
* Authentication and authorization

API and CLI share the same backend logic.

---

## 7. Control Plane Services

Examples:

* Routing daemon
* Interface manager
* Policy engine

Characteristics:

* React to config changes
* Program data plane via kernel APIs

---

## 8. Feature Packages

### 8.1 Package Definition

A package:

* Is an installable unit
* Runs as one or more services
* Declares dependencies
* Declares required capabilities

---

### 8.2 Package Isolation

* Separate process space
* No direct kernel access
* Limited IPC surface

Package crash impact:

* Package-only failure
* No system-wide impact

---

## 9. IPC Design

### 9.1 IPC Contracts

* Versioned
* Typed
* Backward compatible when possible

Breaking changes require:

* New IPC version

---

### 9.2 IPC Patterns

* Request / response
* Publish / subscribe
* Event notification

---

## 10. Logging and Metrics

### 10.1 Logging

* Structured logs
* Per-service log streams
* Centralized log collector

---

### 10.2 Metrics

* Per-service metrics
* Health and performance indicators
* Exportable to monitoring systems

---

## 11. Fault Tolerance

* Service crashes are expected
* Automatic restart with backoff
* Circuit breaker for flapping services

Kernel stability is never affected.

---

## 12. Security Boundaries

* No implicit trust between services
* All actions require capabilities
* Management plane is isolated

---

## 13. Update Behavior

### 13.1 Service Restart

* Config change triggers controlled restart
* Rolling restart where applicable

---

### 13.2 Package Update

* Stop affected services
* Update package
* Restart services
* Rollback on failure

---

## 14. Development Guidelines

* Rust preferred for system services
* Explicit error handling
* No global mutable state

---

## 15. Next Documents

* Orbital OS – Configuration System
* Orbital OS – Package System
* Orbital OS – Networking Data Plane

---

**End of Document**

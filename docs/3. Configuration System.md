# Orbital OS – Configuration System

> **Purpose**
>
> This document defines the **configuration system architecture** of Orbital OS. The configuration system is the **single source of truth** for system state and behavior, designed to be **transactional, atomic, and fault-tolerant**.
>
> A robust configuration system ensures that configuration errors never cause system crashes or prolonged downtime.

---

## 1. Design Principles

* Single source of truth
* Transactional configuration changes
* Atomic apply
* Explicit validation
* Deterministic rollback
* Human- and machine-friendly

Configuration correctness is prioritized over convenience.

---

## 2. Configuration Scope

The configuration system manages:

* Network interfaces
* IP addressing
* Routing and policies
* Firewall and NAT rules
* Feature packages (VPN, hotspot, etc.)
* User accounts and access levels

Runtime state is **not** configuration.

---

## 3. Configuration States

Orbital OS defines three configuration states:

1. **Running Configuration**

   * Actively applied system state
   * Always valid

2. **Candidate Configuration**

   * Editable working copy
   * May be invalid

3. **Committed Configuration**

   * Versioned, persistent configuration

Only committed configurations can become running configurations.

---

## 4. Configuration Lifecycle

```
edit → validate → commit → apply → verify
              ↘ rollback (on failure)
```

---

## 5. Configuration Service

> **Central authority for configuration management**

Responsibilities:

* Maintain candidate configuration
* Perform schema and semantic validation
* Version and persist committed configs
* Coordinate apply and rollback

Other services:

* Read configuration
* Subscribe to configuration changes
* Never modify configuration directly

---

## 6. Configuration Data Model

### 6.1 Structure

* Hierarchical
* Strongly typed
* Explicit defaults

Example (conceptual):

```
interfaces.eth0.address.ipv4
routing.static
firewall.rules
```

---

### 6.2 Schema Definition

Each configuration object defines:

* Type
* Constraints
* Dependencies
* Validation rules

Schema changes are versioned.

---

## 7. Validation Pipeline

Validation occurs in multiple stages:

1. Syntax validation
2. Schema validation
3. Semantic validation
4. Dependency validation
5. Resource availability validation

No partial apply is allowed.

---

## 8. Commit Semantics

A commit:

* Is atomic
* Produces a new config version
* Is immutable once stored

Commit metadata:

* Timestamp
* Author (user/service)
* Change summary

---

## 9. Apply Strategy

### 9.1 Apply Modes

* Immediate apply
* Deferred apply
* Staged apply

---

### 9.2 Service Coordination

* Services receive apply notification
* Each service applies changes independently
* Failure is reported explicitly

Kernel state is modified only after successful validation.

---

## 10. Rollback Strategy

Rollback is:

* Deterministic
* Fast
* Safe

Triggers:

* Apply failure
* Health check failure
* Explicit user request

Rollback restores last known-good configuration.

---

## 11. Concurrency Model

* Single active candidate configuration
* Serialized commit operations
* Concurrent read access allowed

No configuration write conflicts are permitted.

---

## 12. Persistence

Configuration persistence guarantees:

* Atomic write
* Crash consistency
* Power-loss safety

Storage strategy:

* Versioned config store
* Append-only metadata

---

## 13. Auditing and History

Each configuration change is recorded:

* Who changed it
* What changed
* When it changed
* Whether it succeeded

Audit logs are immutable.

---

## 14. Access Control

* Configuration access is capability-based
* Read and write permissions are separated

Examples:

* `config.read`
* `config.write`
* `config.commit`

---

## 15. Error Handling

* Invalid config never affects running system
* Clear error messages
* No silent failure

Errors are reported back to CLI/API consumers.

---

## 16. Integration with Userspace Services

* Services subscribe to config changes
* Services must handle idempotent apply
* Services must support rollback hooks

---

## 17. Update Interaction

* OS updates preserve configuration
* Package updates may introduce new schema
* Backward compatibility preferred

Schema migrations are explicit.

---

## 18. Development Guidelines

* Treat config as code
* Prefer explicit over implicit behavior
* Never auto-correct invalid config

---

## 19. Next Documents

* Orbital OS – Package System
* Orbital OS – Networking Data Plane
* Orbital OS – Management Plane (CLI & API)

---

**End of Document**

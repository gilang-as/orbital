# Orbital OS – Management Plane (CLI & API)

> **Purpose**
>
> This document defines the **management plane architecture** of Orbital OS, including the Command Line Interface (CLI) and programmatic APIs. The management plane is the **only supported entry point** for human operators and automation systems to interact with Orbital OS.

---

## 1. Design Goals

* Single, consistent control interface
* Human-friendly and automation-friendly
* Strong authentication and authorization
* Full auditability
* No direct system mutation outside config system

The management plane never bypasses the configuration system.

---

## 2. Management Plane Overview

```
User / Automation
        ↓
CLI / API Frontend
        ↓
Management Service
        ↓
Configuration Service
        ↓
Control Plane Services
        ↓
Kernel / Data Plane
```

---

## 3. Core Components

### 3.1 Management Service

Responsibilities:

* Central entry point for management actions
* Authentication and authorization enforcement
* Request validation
* Audit logging

The management service contains **no business logic**.

---

### 3.2 Authentication Service

Supported methods:

* Local users
* Token-based authentication
* External identity providers (optional)

Authentication is pluggable and extensible.

---

### 3.3 Authorization (RBAC)

* Role-based access control
* Capability-backed permissions
* Least-privilege by default

Examples:

* `admin`
* `operator`
* `viewer`

---

## 4. Command Line Interface (CLI)

### 4.1 CLI Philosophy

* Predictable command structure
* Explicit verbs
* No hidden side effects

CLI commands modify **candidate configuration only**.

---

### 4.2 Command Structure

```
<resource> <object> <action>

interface eth0 set address 192.168.1.1/24
route add 0.0.0.0/0 via 192.168.1.254
```

---

### 4.3 Configuration Workflow

```
configure
  set ...
  delete ...
  validate
  commit
  rollback
```

---

## 5. API Design

### 5.1 API Principles

* Declarative, not imperative
* Idempotent operations
* Versioned endpoints

---

### 5.2 API Styles

* REST (default)
* gRPC (optional)

Both map to the same backend logic.

---

## 6. Schema and Versioning

* Strongly typed schemas
* Explicit version negotiation
* Backward compatibility preferred

Breaking changes require new API versions.

---

## 7. Audit and Observability

### 7.1 Audit Logs

Each management action records:

* User identity
* Action performed
* Target resource
* Result

Audit logs are immutable.

---

### 7.2 Metrics

* Request rate
* Error rate
* Latency

---

## 8. Multi-User Support

* Multiple concurrent users
* Independent sessions
* Per-session permissions

User activity is fully traceable.

---

## 9. Error Handling

* Clear, actionable error messages
* No silent failures
* Validation errors returned before commit

---

## 10. Security Boundaries

* Management plane isolated from data plane
* No direct kernel access
* All operations capability-checked

---

## 11. Automation and Integration

* CI/CD friendly
* Scriptable via API
* Declarative configuration import/export

---

## 12. Failure Scenarios

Handled cases:

* Auth service failure
* Partial backend failure
* Network disruption

System remains manageable.

---

## 13. Development Guidelines

* Shared backend logic for CLI and API
* Strong typing end-to-end
* Comprehensive integration tests

---

## 14. Next Document

* Orbital OS – Update & Recovery System

---

**End of Document**

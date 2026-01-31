# Orbital OS – Package System

> **Purpose**
>
> This document defines the **package system architecture** of Orbital OS. The package system enables modular feature delivery, independent updates, strong isolation, and safe rollback without compromising system stability.

---

## 1. Design Principles

* Modularity by default
* Independent lifecycle per package
* Strong isolation and least privilege
* Explicit dependencies and capabilities
* Safe install, update, and rollback
* Deterministic behavior

Packages are the primary vehicle for delivering features in Orbital OS.

---

## 2. Package Definition

A **package** is an installable, versioned unit that:

* Provides one or more services
* Declares dependencies on other packages
* Declares required capabilities
* Can be updated or rolled back independently

Packages never execute arbitrary privileged code.

---

## 3. Package Types

### 3.1 System Packages

* Core services (routing, interface manager)
* Shipped with the OS image
* Updated together with OS or as critical updates

---

### 3.2 Feature Packages

Examples:

* WireGuard VPN
* Hotspot
* Advanced firewall

Feature packages are optional and removable.

---

## 4. Package Lifecycle

```
install → verify → enable → run
            ↓
         disable → remove
```

For updates:

```
stop → update → verify → start
             ↘ rollback (on failure)
```

---

## 5. Package Manager Service

> **Central authority for package operations**

Responsibilities:

* Install and remove packages
* Verify package signatures
* Resolve dependencies
* Coordinate service start/stop
* Handle rollback on failure

The package manager never applies configuration directly.

---

## 6. Package Format

Each package contains:

* Manifest file
* Executable binaries
* Optional assets
* IPC interface definitions

---

### 6.1 Package Manifest

The manifest declares:

* Package name and version
* Dependencies
* Required capabilities
* Provided services
* Compatible OS versions

Manifests are machine-validated.

---

## 7. Dependency Management

* Explicit version constraints
* No implicit dependencies
* Dependency graph must be acyclic

Installation fails if dependencies cannot be satisfied.

---

## 8. Capability Assignment

* Capabilities are declared in the manifest
* Granted at service start
* Enforced by kernel and IPC broker

Examples:

* `net.packet.process`
* `net.configure`
* `user.read`

---

## 9. Isolation Model

Each package:

* Runs in its own process space
* Has isolated filesystem access
* Communicates only via IPC

No shared mutable state between packages.

---

## 10. Service Integration

* Package services register with supervisor
* Health checks are mandatory
* Restart policies are defined per service

Service crashes never propagate system-wide.

---

## 11. Update Strategy

### 11.1 Package Update

* Stop affected services
* Apply update
* Restart services
* Verify health

Rollback occurs automatically on failure.

---

### 11.2 Compatibility

* IPC versioning ensures backward compatibility
* Schema migrations are explicit

---

## 12. Rollback Mechanism

Rollback restores:

* Previous package version
* Previous service state

Rollback is:

* Fast
* Deterministic
* Safe

---

## 13. Storage Layout

* Immutable package content
* Versioned package store
* Shared read-only libraries (optional)

---

## 14. Security Model

* Signed packages only
* Signature verification mandatory
* Trusted key store

Unsigned or tampered packages are rejected.

---

## 15. Observability

* Package-level logs
* Per-service metrics
* Install/update audit trail

---

## 16. Failure Scenarios

Handled failures:

* Installation failure
* Dependency conflict
* Service crash loop
* Update failure

Failures never affect kernel stability.

---

## 17. Development Guidelines

* Prefer small, focused packages
* Avoid monolithic packages
* Explicitly declare all dependencies

---

## 18. Next Documents

* Orbital OS – Networking Data Plane
* Orbital OS – Management Plane (CLI & API)
* Orbital OS – Update & Recovery System

---

**End of Document**

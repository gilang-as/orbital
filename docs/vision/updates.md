# Orbital OS – Update & Recovery System

> **Purpose**
>
> This document defines the update and recovery architecture of Orbital OS. The goal is to ensure **safe, atomic, and recoverable updates** for both the core OS and modular packages, without causing system crashes or unrecoverable states.

---

## 1. Design Goals

* Atomic updates (all-or-nothing)
* Always bootable system
* Fast rollback
* Separation between OS and packages
* No panic or hard crash during update failures

---

## 2. Update Types

Orbital OS supports two independent update domains:

### 2.1 Core OS Update

Includes:

* Kernel
* Init system
* Core services
* Management daemon

Characteristics:

* Image-based update
* Requires reboot
* Fully atomic

---

### 2.2 Package Update

Includes:

* Hotspot package
* WireGuard package
* Future feature modules

Characteristics:

* Per-package update
* No reboot required (when possible)
* Versioned and isolated

---

## 3. System Layout (Conceptual)

```
/boot
  ├── slot_A
  └── slot_B

/system
  ├── current -> slot_A
  └── packages
        ├── hotspot
        └── wireguard
```

---

## 4. A/B Slot Update Model

Orbital OS uses an **A/B slot model** for core OS updates.

### 4.1 How It Works

1. System boots from active slot (A)
2. Update is written to inactive slot (B)
3. Bootloader switches next boot to slot B
4. Health checks run after boot
5. If healthy → mark slot B as active
6. If failed → rollback to slot A

---

## 5. Boot Health Verification

After booting into a new slot, the system verifies:

* Kernel boot success
* Management daemon availability
* Core services responsiveness

Failure triggers automatic rollback.

---

## 6. Package Update Model

Packages are:

* Installed into isolated directories
* Versioned
* Activated via symlinks or metadata

### 6.1 Package Lifecycle

```
install → verify → activate → monitor
```

Rollback is per-package.

---

## 7. Update Delivery

Supported update sources:

* Offline image (USB, local file)
* Online update server
* Private mirror

Updates are cryptographically signed.

---

## 8. Verification and Integrity

* Signed update manifests
* Hash verification (SHA-256 or better)
* Version compatibility checks

Unverified updates are rejected.

---

## 9. Failure Handling

Handled scenarios:

* Power loss during update
* Partial download
* Invalid package
* Post-boot service failure

System always returns to last known-good state.

---

## 10. User Interaction

Update actions are performed via:

* CLI
* API

Example (conceptual):

```
orbital system update
orbital system rollback
orbital package update wireguard
```

---

## 11. Logging and Audit

All update actions are logged:

* Who triggered the update
* What was updated
* Result confirmation

Logs are immutable.

---

## 12. Recovery Mode

Orbital OS provides a minimal recovery environment:

* Read-only base system
* CLI access only
* Network optional

Used when both slots fail.

---

## 13. No-Panic Policy

* Update failures must never panic the kernel
* Errors are contained and reported
* System stability has priority over update success

---

## 14. Development Guidelines

* Update logic runs in user space
* Kernel remains update-agnostic
* Extensive fault injection testing

---

## 15. Next Document

* Orbital OS – IPC & API Design

---

**End of Document**

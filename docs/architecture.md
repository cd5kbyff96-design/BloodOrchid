# BloodOrchid Architecture

**Status:** Active System Specification
**Role:** Canonical architecture definition for all implementation decisions
**Authority Level:** Overrides all informal documentation and AI-generated assumptions

---

# 1. System Overview

BloodOrchid is a deterministic, physics-driven simulation system designed to model and transform complex physical fields into structured geometric outputs.

It is not a general-purpose framework. It is a constrained execution pipeline with strict ownership boundaries between computation, state, and transformation.

### Core Principle

```text
Kernel computes physics → Boundary owns state → CVE transforms state → Output consumed externally
```

This chain is **non-bypassable**.

---

# 2. Architectural Invariants (NON-NEGOTIABLE)

The system must always satisfy the following invariants:

### I1 — Deterministic Execution

Identical inputs must produce identical outputs across all layers.

### I2 — Single Source of Truth

The Rust Boundary is the **only authoritative state holder**.

### I3 — Explicit Contracts Only

All cross-layer communication must use versioned protobuf schemas.

### I4 — No Cross-Layer Shortcutting

No module may bypass the Kernel → Boundary → CVE pipeline.

### I5 — Stateless Transformation Rule

CVE must never mutate or persist state.

---

# 3. Core Execution Pipeline

```text
(1) Kernel computes simulation step
        ↓
(2) Kernel emits serialized SimulationState
        ↓
(3) Boundary validates + stores state
        ↓
(4) Boundary exposes immutable snapshot
        ↓
(5) CVE transforms snapshot → GeometryScene
        ↓
(6) Output consumed by CLI / tooling
```

---

# 4. Module Architecture

---

## 4.1 `kernel/` — Physics Computation Layer

### Responsibility

Produces deterministic simulation state.

### Must:

* Implement numerical simulation (PDE/SDE/etc.)
* Generate time-stepped field evolution
* Emit serialized `SimulationState`
* Be fully deterministic

### Must NOT:

* Depend on Rust / CVE / apps
* Perform rendering or visualization
* Store persistent application state
* Introduce runtime nondeterminism

---

## 4.2 `boundary/` — State Authority Layer

### Responsibility

Owns all validated simulation state.

### Must:

* Ingest kernel output
* Validate schema correctness
* Enforce structural invariants
* Store latest accepted snapshot
* Provide immutable access API

### Must NOT:

* Maintain duplicate state systems
* Accept unvalidated or partial data
* Allow external mutation of internal state
* Act as a secondary simulation system

---

## 4.3 `cve/` — Deterministic Transformation Layer

### Responsibility

Convert simulation state into geometry or scene representations.

### Must:

* Accept validated `SimulationState`
* Produce deterministic `GeometryScene`
* Maintain pure functional behavior

### Must NOT:

* Perform physics simulation
* Modify or persist state
* Introduce UI-driven logic
* Contain hidden branching behavior

---

## 4.4 `contracts/` — Cross-Language Schema Layer

### Responsibility

Defines all inter-layer data structures.

### Must:

* Define `SimulationState`
* Define `FieldTensor`
* Define `GeometryScene`
* Be versioned and backward-aware

### Must NOT:

* Be bypassed by ad-hoc structs
* Drift silently across languages
* Be duplicated outside schema definitions

---

## 4.5 `apps/` — Execution & Orchestration Layer

### Responsibility

Human-facing execution entry points.

### Must:

* Trigger kernel execution
* Pass data through boundary and CVE
* Output artifacts or logs
* Remain stateless regarding simulation truth

### Must NOT:

* Become a secondary state system
* Modify kernel or boundary logic
* Short-circuit architecture flow

---

## 4.6 `infra/` — Build & Tooling Layer

### Responsibility

Developer and system infrastructure.

### Must:

* Manage builds and test orchestration
* Support reproducible execution
* Validate system integrity

### Must NOT:

* Alter runtime semantics
* Hide architectural behavior behind scripts
* Override contracts or pipeline flow

---

## 4.7 `tests/` — Verification Layer

### Responsibility

Ensure correctness and determinism.

### Must:

* Validate full pipeline execution
* Test deterministic outputs
* Detect contract drift
* Verify boundary ownership rules

### Must NOT:

* Depend on undefined behavior
* Assume implicit system state
* Ignore architectural constraints

---

# 5. Data Contracts

All system communication depends on strict schema definitions.

### Core Types

* `FieldTensor`
* `SimulationState`
* `GeometryScene`

---

### Contract Rules

* Schemas must be versioned
* Field meaning must remain stable
* Serialization must be deterministic
* Cross-language compatibility is mandatory
* Any schema change = architectural change

---

# 6. Determinism Model

### Guarantee Requirements

The system must ensure:

* Repeatable execution
* Stable output hashes
* No hidden randomness
* Consistent update ordering

---

### Allowed Sources of Variance (strictly controlled)

Only:

* Explicit simulation parameters
* Versioned kernel logic
* Declared configuration inputs

Everything else must be deterministic.

---

# 7. Build & Execution Contract

### Minimum Working System Must:

* Compile without external assumptions
* Execute end-to-end pipeline
* Produce deterministic output artifact
* Be traceable step-by-step

---

### Execution Output Must Include:

* Kernel state generation
* Boundary validation + storage
* CVE transformation output
* Final scene artifact or hash

---

# 8. AI Tooling Governance

AI tools are **assistive execution layers**, not system authorities.

---

### Hard Constraints

AI tools must NEVER:

* Auto-commit code
* Push to remote repositories
* Modify architecture without instruction
* Invent new subsystems
* Skip boundary validation rules

---

### Required Behavior

AI tools MUST:

* Output diffs or full file replacements
* Respect kernel → boundary → CVE flow
* Follow contracts exactly
* Request clarification when schema is ambiguous
* Preserve deterministic behavior assumptions

---

# 9. First Vertical Slice Scope

The initial system must be minimal but complete.

### Included:

* Kernel simulation
* Boundary validation + state storage
* CVE transformation
* CLI execution
* Contract enforcement
* End-to-end deterministic run

### Explicitly excluded:

* Distributed systems
* ML pipelines
* Rendering engines
* Cloud infrastructure
* Plugin ecosystems
* UI frameworks beyond CLI output

---

# 10. Roadmap (Controlled Evolution)

### Phase 1 — MVES (Current)

* Full pipeline working
* Deterministic execution verified

### Phase 2 — Contract Hardening

* Strong schema enforcement
* Cross-language validation tools

### Phase 3 — Visualization Expansion

* Enhanced geometry models
* Multiple CVE mappings

### Phase 4 — Domain Scaling

* Larger simulation domains
* Infrastructure modeling

### Phase 5 — Productionization

* CI/CD integration
* Packaging + deployment strategy

---

# 11. Architectural Summary

BloodOrchid is a strictly layered deterministic simulation system.

Its core identity is defined by:

> Physics generates state.
> State is owned centrally.
> Transformation is pure.
> Output is external.

Any change that violates this flow is architecturally invalid.

Any change that preserves and strengthens this flow is acceptable.

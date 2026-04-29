# BloodOrchid

A deterministic, multi-language simulation monorepo built around a strict kernel → state → transformation pipeline.

---

##  Overview

BloodOrchid is a cross-language computational system designed around a deterministic execution pipeline:

```
C++ Kernel (PDE Simulation)
        ↓
Protobuf Contract Layer
        ↓
Rust Boundary (State Ownership)
        ↓
CVE Core (Deterministic Transformation)
```

The system enforces strict ownership boundaries, reproducibility, and verifiable state transitions.

---

##  System Architecture

Core design is defined in:

* `docs/architecture.md` → system design and constraints
* `docs/contracts.md` → protobuf schema definitions
* `docs/build.md` → build + compilation pipeline
* `docs/roadmap.md` → evolution plan
* `docs/agents.md` → AI tooling governance

---

## ⚙️ Core Components

###  Kernel (C++)

* Deterministic PDE solver
* Produces simulation state frames
* No external dependencies on Rust or CVE

###  Contracts (Protobuf)

* Shared schema across all systems
* Defines:

  * `SimulationState`
  * `FieldTensor`
  * `GeometryScene`

###  Boundary Layer (Rust)

* Owns all simulation state
* Validates kernel output
* Enforces invariants and safety rules

###  CVE Core (Rust)

* Pure transformation layer
* Converts simulation state → geometry representation
* Deterministic, stateless mapping

---

## 🔄 Execution Flow

```
Kernel Output
   ↓
Protobuf Serialization
   ↓
Rust Boundary Ingestion + Validation
   ↓
CVE Transformation
   ↓
Geometry Scene Output
```

---

##  System Constraints

* No ML systems in core pipeline
* No distributed architecture
* No rendering engine dependencies (early stage)
* No implicit cross-language mutation

---

##  AI Tooling Policy

All AI systems (Qwen, Gemini, OpenCode, Ollama, Zed integrations):

* MUST NOT commit automatically
* MUST NOT push to remote
* MUST output diffs or full files only
* MUST respect architecture.md + contracts.md

---

##  Build

See:

```
docs/build.md
```

Primary local execution:

```bash
cargo test --workspace
```

Kernel compilation handled via Rust build pipeline.

---

##  Status

This system is in **MVES (Minimum Viable Execution System)** stage:

* Kernel functional
* State pipeline complete
* CVE transformation operational
* Deterministic execution verified

---

##  Next Milestones

* GPU kernel acceleration layer
* Formal verification of state transitions
* Multi-simulation orchestration layer
* Performance optimization pass

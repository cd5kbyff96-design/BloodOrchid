# BloodOrchid Diagrams

**Document status:** Active system visualization reference  
**Purpose:** Canonical diagrams for architecture, data flow, execution, determinism, tooling, and invariants  
**Audience:** Engineers + AI agents  
**Priority:** Visual truth of system behavior (supports but does not override architecture or contracts)

---

## 1. Purpose

This document defines the structural reality of BloodOrchid.

It complements:
- architecture.md (system rules)
- contracts.md (data definitions)
- decisions.md (why choices exist)

This file defines:
- how the system flows
- how components interact
- how correctness is visually verified

If implementation contradicts these diagrams, implementation is incorrect.

---

## 2. Core System Architecture (Canonical Pipeline)

```text
+-----------+        +-----------+        +-----------+        +-----------+
|  Kernel   | -----> | Boundary  | -----> |   CVE     | -----> |  Output   |
|   C++     |        |   Rust    |        |   Rust    |        |  Apps     |
+-----------+        +-----------+        +-----------+        +-----------+

     Physics              State               Geometry            Presentation
     Simulation           Ownership           Transformation      CLI / Export

---

## 3. Strict Layer Model

[ KERNEL ]
- Numerical simulation
- Deterministic physics
- Time stepping
- Field evolution

        ↓

[ BOUNDARY ]
- Canonical state ownership
- Schema validation
- Snapshot authority
- Contract enforcement

        ↓

[ CVE ]
- State → geometry transformation
- Scene generation
- Deterministic mapping only
- No state ownership

        ↓

[ APPS ]
- CLI / tooling
- Output rendering
- Execution orchestration

---

## 4. Data Flow (Contract Pipeline)

FieldTensor
     ↓
SimulationState
     ↓
Boundary Validation
     ↓
Canonical Snapshot
     ↓
GeometryScene (CVE)
     ↓
Artifact / Output

---

## 5. Execution Lifecycle

[START]
   ↓
Kernel runs deterministic simulation
   ↓
Boundary ingests kernel output
   ↓
Boundary validates against contracts
   ↓
Boundary stores canonical snapshot
   ↓
CVE reads snapshot
   ↓
CVE generates geometry scene
   ↓
App layer emits output/artifact
   ↓
[END] → Deterministic result produced

---

## 6. Determinism Model

Input Parameters
      ↓
Kernel Simulation (fixed timestep)
      ↓
Validated Boundary Snapshot
      ↓
CVE Pure Transformation
      ↓
Output Artifact (hash-stable)

---

## 7. Contract Enforcement Flow

Kernel Output
     ↓
Schema Validation
     ↓
Boundary Check
   ┌──────────────┐
   │ VALID        │ → Store Snapshot
   │ INVALID      │ → Reject + Fail Test
   └──────────────┘
     ↓
CVE Consumes Snapshot
     ↓
Geometry Output

---

## 8. AI Tool Interaction Model

                 Human Operator
                        ↓
     ┌───────────────────────────────────┐
     │                                   │
     v                                   v
Qwen Coder                         Gemini CLI
(Generation)                       (Debug/Analyze)
     \                                   /
      \                                 /
       v                               v
         OpenCode CLI (Refactor Layer)
                     ↓
           Ollama (Second Pass Validation)
                     ↓
              Human Review Gate
                     ↓
           Git Commit / Push (MANUAL ONLY)

---

## 9. Monorepo Structure Map

BloodOrchid/
│
├── kernel/        (C++ physics simulation)
├── boundary/      (Rust state authority)
├── cve/           (Rust geometry engine)
├── contracts/     (protobuf schemas)
├── apps/          (CLI + execution tools)
├── infra/         (build + CI + tooling)
├── tests/         (determinism + integration)
│
├── docs/
│   ├── architecture.md
│   ├── contracts.md
│   ├── build.md
│   ├── roadmap.md
│   ├── agents.md
│   ├── decisions.md
│   └── diagrams.md

---

## 10. Layer Responsibility Matrix

Kernel:
  ✔ physics simulation
  ✔ deterministic computation
  ✖ state ownership
  ✖ rendering

Boundary:
  ✔ canonical state ownership
  ✔ validation
  ✔ snapshot storage
  ✖ physics execution

CVE:
  ✔ geometry generation
  ✔ scene mapping
  ✔ deterministic transforms
  ✖ state mutation
  ✖ simulation logic

Apps:
  ✔ orchestration
  ✔ CLI execution
  ✔ output handling
  ✖ core truth ownership

---

## 11. System Invariants

Determinism
     ↓
No hidden state mutation
     ↓
Strict contract validation
     ↓
Layer isolation enforced
     ↓
Reproducible output guaranteed

---

## 12. Failure Modes (Invalid States)

❌ Kernel writes directly to CVE  
❌ Boundary bypassed by apps  
❌ CVE mutates simulation state  
❌ Contracts drift silently  
❌ AI tools auto-commit changes  
❌ Non-deterministic outputs introduced

---

## 13. Correctness Definition

A valid system run must satisfy:

Kernel produces output
- Boundary validates and owns state
- CVE transforms only validated state
- Output is deterministic
- No layer bypass occurs
- Contracts remain consistent

---

## 14. Summary

BloodOrchid is only valid when:
-simulation is isolated in the kernel
-state is owned exclusively by the boundary
-transformation is pure in CVE
-output is reproducible and traceable
-AI tools operate under strict human control

If the system cannot be drawn by these diagrams, it is not compliant with the architecture.

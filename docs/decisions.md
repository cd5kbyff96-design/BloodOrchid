# BloodOrchid Decisions Log

**Document status:** Active architectural decision record (ADR-style system log)  
**Purpose:** Track all non-trivial engineering, architectural, and tooling decisions in BloodOrchid  
**Audience:** Human maintainers + AI agents  
**Priority:** This document is authoritative for *why the system is the way it is*, not how it is implemented  

---

## 1. Purpose of This Document

This file exists to prevent architectural drift.

It records **why decisions were made**, not just what was implemented.

If `architecture.md` defines the system and `contracts.md` defines the data, then this document defines:

> **why those structures exist in their current form**

Any change that contradicts prior decisions must explicitly reference and supersede them here.

---

## 2. Decision Format Standard

All entries must follow this structure:

### Decision ID Format

ADR-XXXX

Where:
- XXXX is a monotonically increasing integer  
- No reuse or deletion of IDs is allowed  

---

### Required Structure

## ADR-XXXX — Title of Decision

### Status
(Proposed | Accepted | Deprecated | Superseded)

### Context
What problem or constraint triggered this decision.

### Decision
The actual choice made.

### Rationale
Why this decision was chosen over alternatives.

### Consequences
What this decision enables and what it constrains.

### Alternatives Considered
What was rejected and why.

### Impacted Systems
(kernel / boundary / cve / contracts / infra / apps)

### Date
YYYY-MM-DD

---

## 3. Core Architectural Decisions

---

## ADR-0001 — Strict Layered Architecture Enforcement

### Status
Accepted

### Context
Early design iterations showed coupling between simulation logic, state management, and visualization layers, leading to ambiguity in ownership and nondeterministic behavior.

### Decision

Kernel → Boundary → CVE → Output

No bypassing allowed.

### Rationale
Prevents:
- hidden state mutation
- inconsistent simulation results
- cross-layer coupling

### Consequences
+ Strong determinism  
+ Clear ownership boundaries  
- Slight increase in boilerplate  

### Alternatives Considered
- Flexible service mesh architecture (rejected due to nondeterminism risk)  
- Shared global state model (rejected immediately)  

### Impacted Systems
kernel, boundary, cve, contracts  

### Date
2026-04-28  

---

## ADR-0002 — Rust as Boundary State Authority

### Status
Accepted

### Context
Need a deterministic, memory-safe layer to own simulation state after kernel execution.

### Decision
Rust is the canonical owner of all validated simulation state.

### Rationale
- Memory safety guarantees  
- Strong type system  
- Cross-language FFI stability  
- Predictable runtime behavior  

### Consequences
+ Strong contract enforcement layer  
+ Safe ingestion of kernel outputs  
- Requires interop layer complexity with C++  

### Alternatives Considered
- Python (rejected due to nondeterminism and weak typing)  
- C++ (rejected due to unsafe state ownership complexity)  

### Impacted Systems
boundary  

### Date
2026-04-28  

---

## ADR-0003 — CVE Must Be Purely Functional

### Status
Accepted

### Context
Visualization systems often drift into implicit state ownership, breaking reproducibility.

### Decision
CVE must be a pure transformation layer.

### Rationale
Ensures:
- deterministic geometry output  
- traceable transformation pipeline  
- no hidden state mutation  

### Consequences
+ Reproducible visualization  
+ Easier debugging  
- Limits expressive runtime shortcuts  

### Alternatives Considered
- Stateful rendering engine (rejected due to nondeterminism)  
- GPU-driven implicit state model (rejected for traceability loss)  

### Impacted Systems
cve  

### Date
2026-04-28  

---

## ADR-0004 — Protobuf as Cross-Language Contract Backbone

### Status
Accepted

### Context
Multiple languages (C++, Rust, TypeScript, Python) require shared schema definitions.

### Decision
Use Protocol Buffers as the canonical cross-language schema system.

### Rationale
- Mature tooling  
- Strong compatibility guarantees  
- Deterministic serialization  
- Language neutrality  

### Consequences
+ Strong schema enforcement  
+ Cross-language consistency  
- Requires build tooling complexity  

### Alternatives Considered
- JSON schema (rejected due to weak typing and ambiguity)  
- FlatBuffers (rejected due to complexity overhead)  

### Impacted Systems
contracts, kernel, boundary, cve  

### Date
2026-04-28  

---

## ADR-0005 — AI Tools Are Execution Assistants, Not Authorities

### Status
Accepted

### Context
Multiple AI systems (Qwen, Gemini CLI, Ollama, OpenCode, Codex) are used in workflow.

### Decision
AI tools may generate, analyze, and refactor code but cannot execute repository-altering actions autonomously.

### Rationale
Prevents:
- accidental commits  
- architectural drift  
- uncontrolled system changes  

### Consequences
+ Human-controlled system evolution  
+ Safer multi-agent workflow  
- Slight increase in manual overhead  

### Alternatives Considered
- Fully autonomous agent commits (rejected due to instability risk)  

### Impacted Systems
all  

### Date
2026-04-28  

---

## ADR-0006 — Determinism as a First-Class Requirement

### Status
Accepted

### Context
Simulation systems must be reproducible across runs.

### Decision
All simulation outputs must be deterministic given identical inputs.

### Rationale
Enables:
- reproducible debugging  
- scientific validity  
- stable testing  

### Consequences
+ Reliable testing framework  
+ Predictable outputs  
- Restrictions on randomness and concurrency  

### Alternatives Considered
- Probabilistic simulation outputs (rejected for core system)  

### Impacted Systems
kernel, boundary, cve, tests  

### Date
2026-04-28  

---

## 4. Decision Governance Rules

### 4.1 New Decision Requirement

Any of the following requires a new ADR:
- changing system architecture  
- modifying data contracts  
- introducing new build systems  
- altering layer responsibilities  
- changing determinism rules  
- introducing new AI agent roles  

---

### 4.2 Modification Rules

Existing decisions cannot be edited.

They may only be:
- superseded  
- deprecated  
- extended via new ADR  

---

### 4.3 Conflict Resolution

If documents conflict:
1. decisions.md takes precedence for rationale  
2. contracts.md takes precedence for data rules  
3. architecture.md takes precedence for structure  

Conflicts must be explicitly resolved via a new ADR.

---

## 5. AI Tool Compliance Requirement

All AI tools operating in this repository must:
- respect all accepted ADRs  
- avoid introducing behavior that contradicts decisions  
- explicitly flag when a request violates an ADR  
- never silently bypass decision constraints  

---

## 6. Summary

BloodOrchid evolves through explicit decisions, not implicit drift.

Every system behavior must be traceable to:
- an architectural rule  
- a contract definition  
- or a recorded decision  

If it is not recorded here, it is not authoritative.

# BloodOrchid Roadmap

**Document status:** Active roadmap specification  
**Purpose:** Define phased execution plan for building BloodOrchid as a deterministic, contract-driven monorepo system  
**Audience:** Human engineers, AI coding tools, system architects  
**Priority:** Execution guide for sequencing implementation work across a 25–40 person team  

---

## 0. Roadmap Philosophy

BloodOrchid is not built as an accumulation of features. It is built as a **controlled expansion of a deterministic system**.

Every phase must:

- preserve the kernel → boundary → CVE execution model
- maintain strict contract discipline
- avoid architectural drift disguised as “progress”
- remain fully testable at each step

If a phase cannot be validated end-to-end, it is incomplete.

---

## 1. Global Development Constraints

These rules apply to all phases.

### 1.1 Non-negotiable system properties

- Determinism is required (same input → same output)
- Boundary owns all canonical state
- Contracts define all cross-layer communication
- CVE is transformation-only (no simulation authority)
- Kernel is headless and isolated

### 1.2 Prohibited behaviors across all phases

- bypassing contract definitions
- introducing hidden state channels
- allowing CVE or apps to mutate simulation truth
- untracked schema evolution
- implicit serialization formats
- AI tools committing or pushing automatically

### 1.3 AI tooling rule

All AI-assisted development tools must:

- operate in patch/diff mode by default
- never commit without explicit human instruction
- never push to remote repositories
- request clarification when contract ambiguity exists

---

## 2. Phase 1 — Minimum Working Vertical Slice (MVES)

### 2.1 Objective

Prove the full system pipeline works end-to-end with minimal complexity.

This phase is not about scale. It is about **proof of correctness**.

---

### 2.2 Scope

#### Kernel
- deterministic simulation loop
- minimal field evolution (single or small set of tensors)
- reproducible step execution
- structured output emission

#### Boundary
- ingestion of kernel output
- strict schema validation
- canonical state ownership
- snapshot persistence (in-memory or file-backed)

#### CVE
- deterministic transformation of state → geometry
- simple mapping (grid → mesh / primitives)
- no external dependencies on simulation logic

#### Apps / CLI
- single entry execution path
- ability to run full pipeline end-to-end
- output artifact generation (JSON / mesh / debug dump)

#### Contracts
- `FieldTensor`
- `SimulationState`
- `GeometryScene`

#### Tests
- determinism test (same input seed → same output)
- contract validation tests
- full pipeline integration test

---

### 2.3 Success Criteria

- system runs from clean checkout
- full pipeline executes without manual intervention
- kernel output is fully owned by boundary
- CVE does not access kernel directly
- outputs are reproducible byte-for-byte or hash-for-hash

---

## 3. Phase 2 — Contract Hardening

### 3.1 Objective

Make cross-layer communication robust, explicit, and enforcement-driven.

---

### 3.2 Scope

- formal schema generation (protobuf or equivalent)
- strict versioning of all contracts
- contract validation enforcement in boundary layer
- round-trip serialization testing
- explicit rejection of invalid schema versions

---

### 3.3 Success Criteria

- contracts become the only allowed interface between layers
- schema drift is detected at build/test time
- breaking changes are explicit and intentional
- no ad hoc structs exist across layer boundaries

---

## 4. Phase 3 — Build System Stabilization

### 4.1 Objective

Make the monorepo build process deterministic, visible, and scalable.

---

### 4.2 Scope

- unified build orchestration layer
- explicit dependency graph for kernel/boundary/CVE
- CI validation for contract compliance
- reproducible build pipelines
- environment normalization across dev machines

---

### 4.3 Success Criteria

- build produces identical artifacts across environments
- no hidden build-time mutation of system behavior
- dependency graph is explicit and inspectable
- contract violations fail builds immediately

---

## 5. Phase 4 — CVE Expansion

### 5.1 Objective

Expand visualization and geometry expressiveness without violating CVE purity.

---

### 5.2 Scope

- richer geometry primitives (meshes, volumes, fields)
- multiple rendering/representation modes
- improved scene metadata tracking
- export pipelines for downstream tools
- performance improvements in transformation layer

---

### 5.3 Constraints

- CVE remains stateless with respect to simulation truth
- CVE does not introduce or modify canonical state
- all outputs must remain traceable to input snapshot

---

### 5.4 Success Criteria

- geometry fidelity improves without breaking determinism
- CVE remains a pure transformation function
- outputs are reproducible and traceable

---

## 6. Phase 5 — Domain Expansion

### 6.1 Objective

Extend system applicability beyond initial simulation scope.

---

### 6.2 Potential Domains

- infrastructure systems
- structural engineering simulations
- city-scale modeling
- multi-field physics coupling
- higher-order environmental systems

---

### 6.3 Constraints

- all new domains must conform to existing contract system
- no domain introduces new state ownership rules
- kernel remains isolated and deterministic
- boundary remains single source of truth

---

### 6.4 Success Criteria

- new domains plug into existing pipeline without architectural change
- system scaling does not break determinism guarantees
- contracts remain stable across domain additions

---

## 7. Phase 6 — Production Scaling

### 7.1 Objective

Prepare system for large-scale collaboration and long-term maintainability.

---

### 7.2 Scope

- CI/CD hardening and enforcement
- standardized contributor workflows
- packaging and release system
- long-term versioning strategy
- performance benchmarking framework
- observability tooling (non-invasive)

---

### 7.3 Success Criteria

- repository remains maintainable at scale
- onboarding new contributors is deterministic and repeatable
- AI tooling remains constrained by architecture
- system continues to behave predictably under load

---

## 8. Explicit Out-of-Scope Items (All Phases Until Approved)

These are explicitly excluded unless reintroduced via architectural decision:

- distributed systems
- cloud orchestration layers
- frontend-heavy UI systems
- machine learning training pipelines
- plugin ecosystems
- runtime federation systems
- premature optimization of non-critical paths
- ad hoc scripting layers bypassing contracts

---

## 9. Roadmap Governance Rule

Any proposed feature must pass the following test:

- Does it improve determinism?
- Does it clarify ownership?
- Does it strengthen contracts?
- Does it reduce ambiguity?

If the answer is no, the feature is deferred.

If a feature increases complexity without increasing system trustworthiness, it is not allowed in the current phase.

---

## 10. Summary

BloodOrchid is built in phases not to accelerate feature delivery, but to **preserve architectural correctness under growth**.

Each phase is valid only if:

- the system still runs end-to-end
- the contracts still govern all data movement
- the kernel → boundary → CVE pipeline remains intact
- outputs remain reproducible and traceable

If any phase breaks these invariants, the phase is not complete.

# BloodOrchid Contracts

**Status:** Canonical Contract Specification  
**Authority Level:** Defines all cross-layer data structures and serialization rules  
**Scope:** Data only (NOT system architecture or execution flow)  

---

# 1. Purpose

This document defines all **valid, versioned data contracts** used in BloodOrchid.

It strictly governs:

- cross-layer data exchange
- serialization format expectations
- schema versioning rules
- validation requirements
- ownership semantics of data objects

If a structure is not defined here, it is **not a valid system contract**.

---

# 2. Contract Scope Boundary (CRITICAL)

This document does NOT define:

- system architecture
- execution order
- module responsibilities
- runtime behavior

Those belong exclusively to `architecture.md`.

This document defines ONLY:

> What data exists, how it is shaped, and how it must be interpreted.

---

# 3. Canonical Contract Set

The system supports exactly these core contracts in MVES:

## 3.1 FieldTensor

### Definition
A structured numeric field representing simulation or derived physical data.

### Schema Requirements
- Must define:
  - `field_name: string`
  - `field_kind: string`
  - `width: uint32`
  - `height: uint32`
  - `channels: uint32`
  - `cell_spacing: float`
  - `values: float[]`

### Rules
- `values.length == width × height × channels` MUST hold
- values MUST be stored in deterministic row-major order
- all values MUST be finite unless explicitly allowed
- dimensional metadata MUST match actual data layout

### Breaking Change Definition
Any change to:
- field ordering
- dimensional interpretation
- value encoding
is a breaking contract change

---

## 3.2 SimulationState

### Definition
A single authoritative snapshot of kernel output at a given simulation step.

### Schema Requirements
- `simulation_id: string`
- `solver_kind: string`
- `step_index: uint64`
- `simulation_time: float`
- `primary_field: FieldTensor`

### Rules
- MUST be fully decodable without external context
- MUST contain all information required for deterministic CVE transformation
- MUST be self-contained

### Ownership Rule
Once accepted by the Boundary layer:
> SimulationState becomes immutable canonical truth

---

## 3.3 GeometryScene

### Definition
A deterministic geometric representation derived from a validated SimulationState.

### Schema Requirements
- `scene_id: string`
- `source_simulation_id: string`
- `source_step_index: uint64`
- `positions: float[]`
- `indices: uint32[]`
- `value_min: float`
- `value_max: float`

### Rules
- MUST be fully derivable from SimulationState
- MUST NOT introduce new simulation meaning
- MUST remain deterministic for identical inputs
- MUST preserve traceability to source state

---

# 4. Serialization Contract Rules

## 4.1 Deterministic Encoding

All contracts MUST:

- encode deterministically across runs
- preserve field ordering
- preserve numeric precision rules per language binding
- decode back to identical logical structure

---

## 4.2 Allowed Formats

Only explicitly defined formats:

- protobuf (primary)
- generated language bindings from protobuf schema

---

## 4.3 Forbidden Formats

The following are NOT valid contract representations:

- ad hoc JSON as canonical state
- undocumented binary layouts
- struct memory dumps
- implicit language-specific serialization

---

## 4.4 Versioning Rules

A contract version change occurs when:

- field is added/removed
- field meaning changes
- ordering semantics change
- encoding rules change

Non-breaking changes:

- adding optional fields with default-safe behavior

---

# 5. Validation Contract Rules

## 5.1 Validation Requirements

All incoming contract data MUST be validated for:

- structural correctness
- shape consistency
- required field presence
- finiteness of numeric values
- type correctness

---

## 5.2 Rejection Behavior

If validation fails:

- data MUST be rejected
- system MUST NOT proceed with partial state
- error MUST be surfaced to caller or test harness

---

# 6. Determinism Contract Rules

All contracts MUST support deterministic behavior.

This requires:

- stable field ordering
- stable numeric representation rules
- no hidden randomness in serialization
- no environment-dependent decoding behavior

If a field cannot be made deterministic:

- it MUST be explicitly isolated and labeled unstable

---

# 7. Cross-Layer Contract Enforcement

## 7.1 Kernel Output Contract

Kernel output MUST conform to:
- SimulationState schema
- versioned encoding rules

Kernel MUST NOT:
- emit undocumented structures
- rely on downstream inference

---

## 7.2 Boundary Contract Enforcement

Boundary MUST:

- decode all incoming kernel data
- validate against schema
- reject invalid or incomplete payloads
- store ONLY validated canonical state

---

## 7.3 CVE Contract Input Rule

CVE MUST:

- consume ONLY Boundary-approved SimulationState
- treat input as immutable
- NOT reinterpret missing fields
- NOT access kernel output directly

---

# 8. Contract Ownership Model

## Kernel owns:
- raw simulation output generation

## Boundary owns:
- validated canonical state
- decoding + storage

## CVE owns:
- deterministic transformation output

## Apps own:
- presentation and orchestration

No layer may redefine ownership boundaries.

---

# 9. Contract Drift Rule

Any change to:

- field meaning
- schema structure
- serialization behavior

MUST be treated as a **breaking architectural event**, not a refactor.

---

# 10. AI Tool Enforcement Rules

AI tools operating in this repo MUST:

- respect contract definitions exactly
- never invent new fields without instruction
- never bypass schema validation
- always output diffs unless told otherwise
- never modify contracts implicitly
- never push or commit automatically

---

# 11. Minimum Contract Set (MVES)

The minimum viable system ONLY requires:

- FieldTensor
- SimulationState
- GeometryScene

No additional contracts are permitted in MVES unless explicitly introduced.

---

# 12. Relationship to Architecture

- `architecture.md` = system behavior + execution rules
- `contracts.md` = data structure + schema law

If a conflict exists:

> contracts define structure, architecture defines flow

---

# 13. Summary

This document ensures that BloodOrchid has:

- strict data boundaries
- deterministic serialization
- enforceable schema contracts
- no ambiguous cross-layer interpretation

If architecture is the "rules of the system",
then contracts are the "laws of the data itself".

# BloodOrchid Build

**Document status:** Active build specification  
**Purpose:** Define the authoritative build, compile, and execution workflow for the BloodOrchid monorepo  
**Audience:** Engineers, CI systems, AI coding tools  
**Priority:** This document governs how the system is compiled, run, and validated during development  

---

## 1. Build System Overview

BloodOrchid currently uses a **hybrid build model**:

- **Primary build system:** Cargo workspace (Rust)
- **Kernel integration:** C++ kernel compiled and invoked via Rust build pipeline
- **Future system (Bazel):** Present as scaffolding only, not active

The build system is intentionally staged. Only one execution path is considered authoritative for the MVES (Minimum Vertical Execution Slice).

---

## 2. Active Build Architecture

### 2.1 Primary system (authoritative)

The Rust Cargo workspace is the **single active build driver**.

It is responsible for:

- compiling Rust crates
- orchestrating kernel compilation
- running integration tests
- executing CLI entrypoints
- enforcing contract validation (via test layer)

---

### 2.2 C++ Kernel integration

The kernel is treated as a **lower-level compiled component**.

Responsibilities:

- compiled as part of Rust build pipeline
- exposed to Rust via controlled interface (FFI or binding layer)
- produces deterministic simulation output
- does not define its own build orchestration

Constraints:

- kernel must not be independently executed outside build pipeline for MVES
- kernel must not introduce external runtime dependencies
- kernel must remain headless and non-interactive

---

### 2.3 Bazel status (inactive)

Bazel files may exist in the repository.

Important:

- Bazel is **not used in current execution path**
- Bazel is structural scaffolding for future migration phases
- Bazel must not be assumed to be authoritative by tools or contributors

If Bazel and Cargo behavior conflict, Cargo is the source of truth for MVES.

---

## 3. Standard Build Commands

All commands assume execution from the repository root.

---

### 3.1 Validate full workspace

```bash
cargo check --workspace

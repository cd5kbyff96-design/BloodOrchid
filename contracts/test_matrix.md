# Contract Test Matrix

This matrix defines mandatory, CI-enforceable test coverage for Veil Iris.

- Scope: all module interactions defined in `contracts/`.
- Policy: no merge and no deployment without required passing gates.
- Principle: contract-first, boundary-first, invariant-first.

## 1) Repo Mapping (Contract Path -> Test Surface)

- `contracts/shared/*` -> shared compatibility, envelope, error, stream tests.
- `contracts/kernel_boundary/contracts.proto` -> boundary adapter + kernel integration tests.
- `contracts/boundary_invariants/contracts.proto` -> invariant gate validation tests.
- `contracts/boundary_ml/contracts.proto` -> ML interface tests.
- `contracts/boundary_services/contracts.proto` -> service orchestration tests.
- `contracts/services_apps/contracts.proto` -> app/service integration tests.
- `contracts/edge_cloud/contracts.proto` -> distributed sync/orchestration + e2e tests.

Runtime module mapping:

- `boundary/` consumes and enforces all `contracts/*`.
- `kernel/` validated via `kernel_boundary`.
- `invariants/` validated via `boundary_invariants`.
- `ml/` validated via `boundary_ml`.
- `services/` validated via `boundary_services`, `services_apps`, and `edge_cloud`.
- `apps/` validated via `services_apps`.

## 2) Test Layers (Required)

### 2.1 Contract Tests

- **Purpose**: verify schema compatibility and wire-level safety.
- **What is tested**:
  - schema diff rules (additive vs breaking),
  - version bump correctness,
  - reserved tag/enum requirements,
  - backward/forward decode behavior (N-1 <-> N),
  - `ContractError`, `ContractEnvelope`, `StreamControl` conformance.
- **Where it runs**:
  - PR CI on every change under `contracts/`,
  - nightly against full baseline history window.
- **Failure conditions**:
  - breaking change without `major` bump,
  - removed proto field without `reserved`,
  - FlatBuffers id/type mutation in same major,
  - decode failures in backward/forward matrix.

### 2.2 Boundary Tests

- **Purpose**: ensure Rust boundary is the only inter-language transfer layer and enforces contracts.
- **What is tested**:
  - serialization/deserialization through boundary adapters,
  - routing correctness between boundary and each downstream module,
  - cross-language payload normalization,
  - contract error mapping into unified envelope.
- **Where it runs**:
  - PR CI for changes in `boundary/` or `contracts/*`,
  - pre-deploy validation pipeline.
- **Failure conditions**:
  - direct cross-language calls bypassing boundary,
  - mismatch between boundary codecs and contract schemas,
  - missing error mapping to `ContractError`.

### 2.3 Kernel Integration Tests

- **Purpose**: verify deterministic kernel <-> boundary interaction.
- **What is tested**:
  - `KernelStepRequest` / `KernelStepResult` round-trip,
  - deterministic replay from fixed seeds and tick streams,
  - stream control handling for high-frequency state updates.
- **Where it runs**:
  - PR CI for `kernel/`, `boundary/`, `contracts/kernel_boundary/*`,
  - scheduled deterministic replay suite.
- **Failure conditions**:
  - non-deterministic replay outputs,
  - state vector decoding mismatch,
  - contract-incompatible command payload.

### 2.4 Invariant Validation Tests

- **Purpose**: guarantee OCaml invariant gate correctness before state propagation.
- **What is tested**:
  - boundary -> invariants evaluation request/response compatibility,
  - acceptance/rejection semantics and violation payload shape,
  - escalation behavior for repeated invariant failures.
- **Where it runs**:
  - PR CI for `invariants/`, `boundary/`, `contracts/boundary_invariants/*`,
  - pre-merge mandatory gate.
- **Failure conditions**:
  - invalid or unparseable invariant inputs,
  - false accept on known violating fixtures,
  - missing violation detail fields.

### 2.5 ML Interface Tests

- **Purpose**: validate boundary <-> ML contract integrity for inference/training signals.
- **What is tested**:
  - feature batch encoding schema compatibility,
  - inference request/response envelope correctness,
  - training signal schema and label payload compatibility,
  - timeout and error propagation behavior.
- **Where it runs**:
  - PR CI for `ml/`, `boundary/`, `contracts/boundary_ml/*`,
  - nightly cross-model compatibility runs.
- **Failure conditions**:
  - tensor/feature payload decoding failures,
  - missing model/version metadata,
  - response schema mismatch with declared contract.

### 2.6 Service Orchestration Tests

- **Purpose**: validate distributed contract behavior across services and edge/cloud sync.
- **What is tested**:
  - publish/subscribe contract conformance (`boundary_services`),
  - services/apps command/query/projection contract conformance (`services_apps`),
  - edge/cloud sync and reconciliation contract conformance (`edge_cloud`),
  - stream cursor, ack, and retry semantics under partial failure.
- **Where it runs**:
  - PR CI for `services/`, `apps/`, `contracts/boundary_services/*`, `contracts/services_apps/*`, `contracts/edge_cloud/*`,
  - staging environment orchestration suite.
- **Failure conditions**:
  - out-of-order or invalid cursor progression,
  - unhandled distributed retries/timeouts,
  - schema drift between producer and consumer services.

### 2.7 End-to-End System Tests

- **Purpose**: validate complete staged topology from kernel through apps with contracts enforced.
- **What is tested**:
  - kernel -> boundary -> invariants -> ML -> services -> apps flow,
  - edge-cloud synchronization and convergence,
  - contract error handling under injected failure scenarios,
  - deterministic behavior and replay integrity for approved scenarios.
- **Where it runs**:
  - post-merge mainline CI,
  - pre-production deployment gate.
- **Failure conditions**:
  - any contract violation in system trace,
  - invariant gate bypass,
  - non-deterministic replay for deterministic scenarios,
  - unresolved reconciliation conflicts above threshold.

## 3) CI Gating Rules

### 3.1 Must Pass Before Merge

The following are strict merge blockers:

1. Contract tests for all touched contract families.
2. Boundary tests for any change touching `boundary/` or `contracts/*`.
3. Layer-specific tests for touched module(s):
   - `kernel/` changes -> kernel integration tests,
   - `invariants/` changes -> invariant validation tests,
   - `ml/` changes -> ML interface tests,
   - `services/` or `apps/` changes -> service orchestration tests.
4. Static dependency graph check:
   - no skipped layers,
   - no circular dependencies,
   - no hidden cross-language coupling.
5. Version/compatibility policy checks from `contracts/COMPATIBILITY.md`.

Merge must fail-fast on first contract policy violation.

### 3.2 Blocks Deployment

The following block promotion to production:

1. Any failing end-to-end system test.
2. Any failing deterministic replay check.
3. Any failing invariant validation suite.
4. Any unresolved contract compatibility regression between staged and production artifacts.
5. Any unapproved `major` contract change without migration completion evidence.

Deployment must remain frozen until all blockers are green.

## 4) Execution Scope by Pipeline Stage

- **PR Pipeline**:
  - contract tests,
  - boundary tests,
  - impacted layer tests.
- **Mainline Pipeline**:
  - full contract matrix,
  - full integration across modules,
  - deterministic replay validation.
- **Pre-Deploy Pipeline**:
  - full end-to-end suite,
  - staging orchestration and edge-cloud reconciliation,
  - release artifact compatibility verification.

No stage may bypass previous stage outcomes.

## 5) Enforceable CI Implementation Requirements

CI configuration must implement these explicit checks:

1. Path-based test selection with conservative expansion:
   - touching shared contracts triggers all downstream contract tests.
2. Contract baseline comparison against main branch artifacts.
3. Cross-language code generation and compile/import checks:
   - Rust, C++, Python, OCaml, Elixir.
4. N-1 and N version wire compatibility test matrix.
5. Deterministic replay checksum comparisons for kernel paths.
6. Invariant failure escalation counter gate:
   - repeated safety failures automatically fail pipeline.
7. Artifact provenance checks:
   - tests must run on the same built contract artifacts promoted to deployment.

If any required check is missing from CI config, the pipeline is non-compliant.

## 6) Ownership and Accountability

- Contract owners (planner/integrator role) approve contract and compatibility changes.
- Module owners approve implementation-side adapter changes.
- Verifier role can veto merge on safety, drift, or compatibility failures.

No override is allowed for failing mandatory contract gates.

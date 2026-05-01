# Invariants - OCaml Structural Contracts

## Purpose
Tri-state gate for structural contract enforcement.

## Gate States
- **Valid** - Constraint satisfied
- **Uncertain** - Requires human review
- **Invalid** - Constraint violated

## Validators
- `causal.ml` - Monotonicity, counterfactual fairness, SURD balance
- `topological.ml` - Persistence stability, equivalence class consistency
- `policy.ml` - Control policy constraints
- `federation.ml` - Pre-admission filter for federated updates

## Deployment
Runs as **sidecar container** alongside each boundary pod:
- Sub-millisecond IPC (not network hop)
- OCaml → Rust via generated C-ABI bindings

## On Invalid
Rust loads last-known-good policy from TimescaleDB + fires escalation.

## Build
```bash
cd invariants
dune build
```
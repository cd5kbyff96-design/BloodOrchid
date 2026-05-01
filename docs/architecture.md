# Arrakis Architecture

## System Overview
Multi-language monorepo for physics simulation, causal ML, and infrastructure monitoring.

## Language Boundaries
- **C++/CUDA** (`kernel/`) - Physics engine, PDE/SDE solvers
- **Rust** (`boundary/`, `federation/`, `edge/`) - Inter-language bridge, consensus, edge runtime
- **Python** (`causal/`, `studio/`, `hyperlattice/`, `echoinfra/`) - TDA, causal ML, products
- **OCaml** (`invariants/`) - Structural contract enforcement
- **Elixir** (`federation/`) - Quorum coordination
- **TypeScript** - Frontend applications
- **SQL** (`db/`) - TimescaleDB schemas

## Core Principles
1. Schema-first contracts in `contracts/`
2. Rust is the ONLY inter-language transfer layer
3. OCaml gate runs as sidecar (sub-ms IPC)
4. No cross-language implicit imports

## Directory Structure
```
contracts/     - Proto/FlatBuffers schemas (source of truth)
kernel/        - C++17/CUDA physics source of truth
boundary/      - Rust inter-language bridge
causal/        - Python TDA + causal ML
invariants/    - OCaml structural contracts
federation/    - Rust consensus + Elixir quorum
edge/          - Rust certified edge runtime
db/            - SQL/TimescaleDB schemas
studio/        - Product: Architecture + BIM
hyperlattice/  - Product: City intelligence
echoinfra/     - Product: Infrastructure monitoring
infra/         - K8s, Terraform, Ray, CI
docs/          - Documentation
```
# Contracts - Schema-First Design

## Overview
All inter-module communication is defined here. This is the source of truth for data structures.

## Supported Formats
- **Protocol Buffers** (.proto) - Primary format
- **FlatBuffers** - Performance-critical hot paths

## Directory Structure
```
contracts/
├── shared/           - Common types (envelope, errors, streaming, versioning)
├── kernel_boundary/  - Kernel ↔ Boundary contract
├── boundary_ml/       - Boundary ↔ ML (causal)
├── boundary_invariants/ - Boundary ↔ OCaml gate
├── boundary_services/ - Publish/Subscribe contracts
├── services_apps/    - App command/query contracts
└── edge_cloud/       - Edge ↔ Cloud sync
```

## Breaking Change Policy
1. Never delete fields (use reserved)
2. Adding optional fields is safe
3. Breaking change = new message name + major version

## Testing
All contracts must have corresponding tests in `contracts/test_matrix.md`.
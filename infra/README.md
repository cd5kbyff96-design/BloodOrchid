# Infrastructure - Build & Deployment

## Structure
```
infra/
├── k8s/           - Kubernetes manifests
│   ├── boundary/  - Rust boundary pod
│   ├── causal/    - Python/Ray workers
│   ├── federation/ - Rust + Elixir cluster
│   ├── ocaml-gate-sidecar/ - OCaml as sidecar
│   └── helm/      - Helm charts
├── ray/           - Ray cluster config
├── terraform/     - Cloud provisioning
└── ci/            - CI pipelines
```

## CI Pipelines
- `contracts.yml` - Schema diff gate
- `boundary.yml` - Round-trip tests
- `kernel.yml` - Deterministic replay
- `invariants.yml` - Gate validation
- `integration.yml` - Cross-module suite
- `deploy.yml` - Pre-prod gate
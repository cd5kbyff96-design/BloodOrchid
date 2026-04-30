Purpose
This file explains the planned top-level layout and conventions for the repo-reorg/planning branch.

Top-level layout
- build/         — repo-level build orchestration (Bazel or Pants bootstrap)
- infra/         — k8s/terraform/helm and deployment artifacts
- services/      — deployable services (each service has Dockerfile, infra/, tests)
- libs/          — shared libraries organized by language:
  - libs/rust/   — Cargo workspace under libs/rust/crates
  - libs/elixir/ — mix umbrella apps under libs/elixir/apps
  - libs/python/ — pyproject-based packages under libs/python
- sims/          — simulation projects
- models/        — model metadata (NOT binary blobs)
- data/          — data manifests and migration scripts
- sql/           — PLpgSQL and migration scripts
- tools/         — repo tooling and scripts
- docs/          — architecture and runbooks
- notebooks/     — exploratory notebooks (pin to commits)
- samples/       — quickstart examples
- experiments/   — short-lived research code (defined retention)
- scripts/       — repo maintenance helpers

Developer rules (short)
- Use small, per-package commits when moving code.
- Preserve history with git mv when possible.
- Add a Cargo workspace entry for any moved Rust crates.
- Add entry to CODEOWNERS for new areas.
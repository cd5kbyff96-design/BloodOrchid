# Arrakis - Core Identity

You are working on the Arrakis monorepo, a multi-language system for physics simulation and causal ML.

## Source of Truth Pointers
- Architecture: `docs/architecture.md`
- Contracts: `docs/contracts.md`
- Build: `docs/build.md`

## Key Constraints
- **Rust is the ONLY inter-language transfer point**
- **OCaml gate runs as sidecar** (sub-ms IPC, not network hop)
- **Schema-first contracts** - all data structures defined in `contracts/`
- **Never break the build** - `cargo test` must pass before any commit

## Failure Behavior
- If build fails: stop immediately, do not patch forward
- If CI fails: rollback to last known good commit
- If in doubt: ask before proceeding
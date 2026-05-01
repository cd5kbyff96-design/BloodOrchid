# Test Protocol

## Required Test Layers

### Unit Tests
- Each crate has `tests/` subdirectory
- Run with `cargo test -p <crate-name>`

### Integration Tests
- `tests/integration/` - Cross-module flows
- Run with `tests/integration/run_all.sh`

### Test Coverage Gates
| Module | Required Tests |
|--------|----------------|
| kernel/ | Determinism, numerical sanity, registry dispatch |
| boundary/ | Strict blocking, version mismatch, unknown fields |
| cve/core/ | Roundtrip, shape validation, hash stability |
| invariants/ | Valid/Uncertain/Invalid states |
| causal/ | TDA homology, SURD decomposition |

## Verification Checklist
- [ ] `cargo test --workspace` passes
- [ ] No new clippy warnings
- [ ] Integration tests pass
- [ ] Documentation builds (if changed)

## Mandatory Checks (Never Skip)
```bash
cargo clippy --workspace -D warnings
cargo fmt --check
cargo test --workspace
```

## Failure Protocol
1. Stop immediately - do not patch forward
2. Revert to last green commit
3. Fix issue in isolation
4. Re-run full test suite
5. Only then commit
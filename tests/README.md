# Tests

## Structure
```
tests/
├── integration/   - Cross-module end-to-end tests
├── kernel/        - Kernel-specific tests (determinism, numerical sanity)
└── invariants/    - OCaml invariant validation tests
```

## Running Tests

### All tests
```bash
cargo test --workspace
```

### Specific module
```bash
cargo test -p boundary-runtime
cargo test -p cve-core
```

### Integration
```bash
./tests/integration/run_all.sh
```

## Test Standards
- Unit tests live inside each crate (Rust: `tests/` subdir, C++: `kernel/tests/`)
- Integration tests are strictly cross-module
- See `.cursor/rules/03-test-and-verification.md` for required test gates
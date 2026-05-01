# Build Order

## Mandatory Build Sequence

### Phase 1: Foundation
1. `contracts/` - Build first, no dependencies
2. `kernel/` - C++17/CUDA, produces static library `libveiliris_kernel.a`

### Phase 2: Bridge
3. `boundary/` - Rust crates, links kernel via FFI

### Phase 3: Validation
4. `invariants/` - OCaml, compiles after boundary is stable
5. `cve/core/` - Depends on boundary-runtime

### Phase 4: Products
6. `federation/` - Rust consensus + Elixir quorum
7. `causal/` - Python TDA + causal ML
8. `edge/` - Rust edge runtime

### Phase 5: Applications
9. `studio/`, `hyperlattice/`, `echoinfra/` - Product frontends
10. `apps/` - CLI tools

## Build Commands
```bash
# Full Rust build
cargo build --workspace

# Kernel only
cd kernel && cmake -B build && cmake --build build

# Run tests
cargo test --workspace
```

## Never Do
- Never `cargo build` while another build is running
- Never skip `cargo test` before commit
- Never modify contracts/ without updating test_matrix.md
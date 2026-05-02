# Phase 8: Performance & Kernel Acceleration Layer

## Design Document

---

## 1. OPTIMIZATION STRATEGY BY SUBSYSTEM

### 1.1 Kernel (C++)

| Optimization | Current State | Target | Safety Level |
|--------------|---------------|--------|---------------|
| Loop unrolling | Manual | Compiler auto-vectorization | SAFE |
| Memory layout | Row-major | SoA (Structure of Arrays) for hot paths | SAFE |
| SIMD operations | None | AVX2/AVX-512 via compiler flags | SAFE |
| Branch prediction | Implicit | Pragma hints for hot paths | SAFE |
| Buffer reuse | Per-call allocation | Thread-local scratch buffers | SAFE |
| Inlining | Default | Explicit inline for hot functions | SAFE |

**Key Constraint**: Output MUST be byte-identical to scalar baseline.

### 1.2 Boundary (Rust)

| Optimization | Current State | Target | Safety Level |
|--------------|---------------|--------|---------------|
| Protobuf parsing | Default | `protox` with zero-copy | SAFE |
| Allocation | std::vec | SmallVec for fixed-size buffers | SAFE |
| Serialization | Default | `serde` with `iterex` | SAFE |

**Key Constraint**: Boundary is state authority - no performance changes may affect state semantics.

### 1.3 CVE (Rust)

| Optimization | Current State | Target | Safety Level |
|--------------|---------------|--------|---------------|
| Iterator chains | Manual | ExactSizeIterator hints | SAFE |
| SIMD for geometric ops | None | Portable SIMD (stdsimd) | SAFE |
| Num geometry | f64 | f32 where precision allows | CAUTION |

**Key Constraint**: Pure transformation - any optimization must preserve exact mathematical equivalence.

### 1.4 CLI (Rust)

| Optimization | Current State | Target | Safety Level |
|--------------|---------------|--------|---------------|
| Task spawning | Sequential | Parallel for independent ops | SAFE |
| Async I/O | None | Tokio for non-blocking | SAFE |

**Key Constraint**: Orchestration only - performance gains must not introduce state coupling.

---

## 2. DETERMINISTIC PARALLEL EXECUTION MODEL

### 2.1 Thread Safety Boundaries

```
┌─────────────────────────────────────────────────────────┐
│                    CLI (Orchestration)                   │
│  - Spawns parallel tasks via tokio                      │
│  - No shared mutable state between tasks                 │
│  - Joins deterministically (ordered)                     │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│              Boundary (State Authority)                 │
│  - Single-threaded execution                             │
│  - Mutex only for thread-safe FFI boundary              │
│  - Snapshots provide deterministic ordering              │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                 CVE (Pure Transformer)                   │
│  - Stateless parallel transforms                        │
│  - Rayon for embarrassingly parallel ops                │
│  - Reduction order: deterministic (sorted indices)       │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                Kernel (Deterministic)                    │
│  - Single-threaded reference (guaranteed determinism)   │
│  - Parallel path only via:                               │
│    a) Deterministic reduction (commutative + associative)│
│    b) SIMD (bit-exact across runs)                       │
│    c) GPU with deterministic scheduling                   │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Parallel Rules

1. **No shared mutable state** - All parallelism uses functional patterns
2. **Deterministic ordering** - Reductions use sorted keys, not hash iteration
3. **Commutative operations only** - Addition, min, max are safe; multiplication order matters
4. **No thread-local random state** - All RNG seeded from deterministic source
5. **Barrier synchronization** - Explicit ordering at boundaries

### 2.3 CVE Parallel Strategy (Rayon)

```rust
// Deterministic: sort indices first, then parallel transform
let mut indices: Vec<usize> = (0..positions.len()).collect();
indices.sort(); // Explicit ordering guarantee

positions
    .par_iter()
    .zip(indices.par_iter())
    .map(|(pos, &idx)| transform_element(pos, idx))
    .collect()
```

### 2.4 CLI Async Strategy (Tokio)

```rust
// Parallel but ordered join
let results: Vec<_> = tokio::task::JoinSet::new()
    .spawn(async { /* task 1 */ })
    .spawn(async { /* task 2 */ })
    .join_all()
    .await; // Deterministic completion order
```

---

## 3. GPU ACCELERATION CONSTRAINTS

### 3.1 CUDA Scaffold Status

| Component | Status | Location |
|-----------|--------|----------|
| CUDA launch config | Stub | `kernel/cuda/cuda_launch.hpp` |
| Noop kernel | Stub | `kernel/cuda/cuda_support_stub.cpp` |
| Build config | Exists | `kernel/cuda/BUILD.bazel` |

### 3.2 GPU Execution Constraints

1. **Deterministic scheduling required**
   - CUDA streams must have deterministic ordering
   - No non-deterministic kernel fusion
   - Explicit synchronization points between kernels

2. **Bit-exact output requirement**
   - GPU float precision must match CPU reference
   - Use `__nv_fp32` equivalent for compatibility
   - Disable relaxed floating point (`--prec-div=false`)

3. **Memory model**
   - Unified memory preferred (managed)
   - Explicit copy-in/copy-out for boundary interaction
   - No asynchronous memory operations without synchronization

4. **Fallback path mandatory**
   - CPU reference implementation always available
   - GPU failures must fallback to CPU deterministically
   - Test both paths continuously

### 3.3 GPU Optimization Roadmap

| Phase | Feature | Determinism Strategy |
|-------|---------|---------------------|
| Phase 8.1 | SIMD via compiler flags | AVX2 - bit-exact by default |
| Phase 8.2 | Thread-parallel heat solver | Commutative reduction |
| Phase 8.3 | CUDA integration | Deterministic stream ordering |
| Phase 8.4 | Multi-GPU support | Deterministic work partitioning |

### 3.4 CUDA Code Constraints

```cpp
// SAFE: Deterministic GPU pattern
__global__ void heat_kernel(float* current, float* next, int width, int height) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= width * height) return;

    // Deterministic index ordering - no race condition
    // Writes to unique indices only
    next[idx] = compute_laplacian(current, idx, width);
}

// UNSAFE: Non-deterministic pattern
__global__ void unsafe_kernel(float* data) {
    atomicAdd(&data[0], 1.0f); // Non-deterministic order
}
```

---

## 4. PERFORMANCE REGRESSION TESTING STRATEGY

### 4.1 Test Categories

| Category | Frequency | Pass Criteria |
|----------|-----------|---------------|
| Determinism check | Every run | SHA-256 matches baseline |
| Microbenchmark | Per commit | ±5% of baseline |
| Integration benchmark | Daily | Throughput > baseline |
| Regression suite | Weekly | All tests pass |

### 4.2 Benchmark Infrastructure

```rust
// benchmark/determinism.rs
fn kernel_determinism_benchmark(c: &mut Criterion) {
    c.bench_function("kernel_8_steps", |b| {
        b.iter(|| {
            let result = KernelBridge::run_heat(8);
            // Verify determinism on every iteration
            assert_hash_matches_baseline(&result);
        })
    });
}

fn verify_determinism(output: &[u8]) {
    let hash = sha256(output);
    let baseline = load_baseline_hash();
    assert_eq!(hash, baseline, "Determinism violation detected");
}
```

### 4.3 Regression Detection

1. **Output hash regression** - Any change in SHA-256 = FAIL
2. **Performance regression** - >10% slowdown = INVESTIGATE
3. **Memory regression** - >20% increase = INVESTIGATE
4. **CPU utilization regression** - Platform-dependent, track trend

### 4.4 CI Integration

```yaml
# .github/workflows/performance.yml
- name: Determinism Check
  run: scripts/check_determinism.sh

- name: Microbenchmarks
  run: cargo bench -- --save-baseline

- name: Regression Comparison
  run: cargo bench -- --compare baseline
```

---

## 5. SAFE VS UNSAFE OPTIMIZATION BOUNDARIES

### 5.1 Safe Optimizations (Always Allowed)

| Category | Examples | Justification |
|----------|----------|---------------|
| Compiler flags | `-O3`, `-march=native`, `AVX2` | Bit-exact by standard |
| Data layout | SoA, cache alignment | Memory view unchanged |
| Iterator hints | `.size_hint()`, `ExactSizeIterator` | Semantic preserving |
| SmallVec | Inline small buffers | Allocation behavior unchanged |
| SIMD intrinsics | `std::simd` | Bit-exact when used correctly |

### 5.2 Conditional Optimizations (Requires Verification)

| Category | Examples | Verification Required |
|----------|----------|----------------------|
| Floating-point precision | f64 → f32 | Full test suite pass |
| Approximation | `sin(x) ≈ x` for small x | Numerical equivalence proof |
| Algorithmic change | Iterative → closed-form | Determinism test |
| Parallelization | Sequential → parallel | Regression test |

### 5.3 Prohibited Optimizations

| Category | Reason | Alternative |
|----------|--------|--------------|
| Random optimization | Breaks determinism | Seed deterministic RNG |
| Non-deterministic sorting | Hash iteration order | Explicit sort |
| Lock-free structures | Race conditions | Single-threaded or barrier |
| Approximate math | Floating-point drift | Bit-exact operations |
| Lazy initialization | Order-dependent | Eager initialization |

### 5.4 Unsafe Code Boundaries

```
┌─────────────────────────────────────────────────────────┐
│                    SAFE ZONE                            │
│  - Rust safe code                                        │
│  - C++ with -O2/-O3 (deterministic)                      │
│  - SIMD via portable abstractions                        │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                 BOUNDARY (FFI)                           │
│  - unsafe { } blocks explicitly marked                  │
│  - No unsafe in CVE (pure)                              │
│  - Minimal unsafe in boundary (FFI only)                │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                 RESTRICTED ZONE                          │
│  - Kernel FFI only                                      │
│  - CUDA interop only                                    │
│  - All unsafe code in kernel/ffi/                       │
└─────────────────────────────────────────────────────────┘
```

### 5.5 Review Checklist for Optimizations

- [ ] Output SHA-256 unchanged?
- [ ] No new `unsafe` blocks added?
- [ ] Test suite passes 100%?
- [ ] Determinism test passes 10+ iterations?
- [ ] Performance delta documented?
- [ ] Regression baseline updated?

---

## 6. IMPLEMENTATION ROADMAP

### Phase 8.1: Immediate (1-2 weeks)

- [ ] Add compiler flags: `-O3 -march=native -ffast-math` (verify deterministic)
- [ ] Implement SoA layout for heat solver
- [ ] Add Rayon parallel transforms to CVE
- [ ] Set up microbenchmark infrastructure
- [ ] Establish determinism baseline

### Phase 8.2: Short-term (1 month)

- [ ] Implement CUDA heat solver (with CPU fallback)
- [ ] Add SIMD via portable SIMD crate
- [ ] Implement CLI async pipeline
- [ ] Full regression test suite
- [ ] Performance dashboard

### Phase 8.3: Medium-term (2-3 months)

- [ ] Multi-GPU support with deterministic work distribution
- [ ] Advanced PDE solvers (Navier-Stokes)
- [ ] Memory optimization (custom allocators)
- [ ] Integration benchmarks

---

## 7. RISK MITIGATION

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Determinism break | Medium | Critical | Always verify SHA-256 before/after |
| GPU fallback failure | Low | High | CPU path always works |
| Performance regression | Medium | Medium | Continuous benchmarking |
| Precision loss (f32) | Medium | Medium | Test with f64 baseline |
| SIMD portability | Low | Low | Use portable-simd crate |

---

## 8. SUCCESS CRITERIA

| Metric | Current | Phase 8 Target |
|--------|---------|-----------------|
| Kernel throughput | Baseline | 2-5x improvement |
| End-to-end latency | Baseline | 1.5-3x improvement |
| Determinism | Verified | Verified + regression test |
| Test coverage | 100% | 100% + benchmark |
| GPU path | Stub | Functional with fallback |

---

**DESIGN APPROVAL**

- [ ] All subsystem leads agree on optimization boundaries
- [ ] Determinism verification strategy validated
- [ ] GPU constraints documented and agreed
- [ ] Regression testing strategy approved
- [ ] Unsafe code boundaries confirmed

---

*End of Phase 8 Design*
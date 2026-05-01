# Kernel - C++17/CUDA Physics Engine

## Structure
```
kernel/
├── include/veiliris/kernel/
│   ├── core/          - Status, types, tensor, field_state
│   ├── pde/           - PDE solvers (heat, navier_stokes, etc.)
│   ├── sde/           - SDE solvers (euler_maruyama)
│   ├── lyapunov/      - Benettin algorithm (CUDA)
│   ├── execution/     - KernelEngine, ExecutionPlan
│   └── ffi/          - C-ABI surface (kernel_c_api.h)
├── src/              - Implementation
├── tests/            - Unit tests
└── CMakeLists.txt   - Build config (cmake >= 3.24)
```

## Build
```bash
cd kernel
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

## FFI Contract
The kernel exposes a C ABI in `include/veiliris/kernel/ffi/kernel_c_api.h`:
- `veiliris_kernel_step()` - Execute one simulation step
- Request/Result structures for state transfer

## Solver Registry
All solvers are registered in `src/registry/solver_registry.cpp`:
- Missing solver key → `kRegistryMiss`
- PDE solvers: heat2d, navier_stokes, elasticity, etc.
- SDE solvers: euler_maruyama

## Determinism
- ReplayRecorder captures checkpoints
- Xorshift64 RNG is seed-deterministic
- Identical seed → identical hash across runs
# Codex Agent Pack

## Role
You are assisting with low-level implementation tasks.

## Focus Areas
- Kernel C++ implementation
- Rust FFI bindings
- Performance-critical paths
- Memory management

## Guidelines
- C++17 standard minimum, CUDA where applicable
- No external runtime deps in kernel/
- Use veiliris_kernel C ABI for all kernel calls
- Validate determinism: same seed → same output

## Testing
- Unit tests in kernel/tests/
- Determinism tests mandatory
- Numerical sanity checks for all solvers
# Boundary - Rust Inter-Language Transfer Layer

## Purpose
Boundary is the ONLY inter-language transfer point in the system.

## Crates
- `boundary-core` - Contract ingestion, validation, routing
- `boundary-serialize` - FlatBuffers/Cap'n Proto hot-path
- `boundary-stream` - Tokio async streaming + SLO backpressure
- `boundary-ffi` - C-ABI (kernel) + PyO3 (Python) bindings
- `boundary-router` - Semantic output routing
- `boundary-monitor` - SLO telemetry → TimescaleDB
- `boundary-export` - IFC4/USD writers
- `boundary-cve` - Geometry transformation (CVE)

## SLO Classes
- HardRT (<10ms) - Real-time simulation
- CitySync (<50ms) - City intelligence
- MLInference (<250ms) - Causal ML inference
- Federated (<2s) - Federated updates

## FFI Surface
```rust
// From Python (via PyO3)
fn get_causal_snapshot() -> SimulationState;
fn push_ml_features(features: FeatureBatch);

// From C++ (via C-ABI)
fn veiliris_kernel_step(request: KernelStepRequest) -> KernelStepResult;
```

## Streaming
- Explicit backpressure policies
- Tokio async + blocking modes
- Drop policy on timeout
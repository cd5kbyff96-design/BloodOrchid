# Language Boundaries

## Explicit Separation
Each language lives in its own directory tree:
- C++/CUDA → `kernel/`
- Rust → `boundary/`, `federation/`, `edge/`, `apps/`
- Python → `causal/`, `studio/`, `hyperlattice/`, `echoinfra/`
- OCaml → `invariants/`
- Elixir → `federation/`
- TypeScript → `*/frontend/`
- SQL → `db/`

## Inter-Language Rules
1. **Python cannot write simulation state directly** - goes through boundary/ffi PyO3
2. **OCaml gate is a sidecar** - co-located with boundary pod, not a remote service
3. **All kernel calls go through Rust FFI** - never call C++ directly from Python/Elixir
4. **No implicit cross-language imports** - always explicit via boundary/contracts

## Boundary Crates
- `boundary-core` - Contract ingestion, validation, routing
- `boundary-serialize` - FlatBuffers/Cap'n Proto serialization
- `boundary-stream` - Tokio async streaming + backpressure
- `boundary-ffi` - C-ABI (kernel) and PyO3 (Python) bindings
- `boundary-router` - Semantic output routing by SLO class
- `boundary-monitor` - SLO telemetry → TimescaleDB
- `boundary-export` - IFC4/USD writers
- `boundary-cve` - Geometry transformation
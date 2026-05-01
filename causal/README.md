# Causal - Python TDA + Causal ML

## Structure
```
causal/
├── tda/           - Topological Data Analysis (GUDHI/Ripser)
│   ├── homology.py    - Persistent homology (H0, H1, H2)
│   ├── mapper.py      - Mapper construction
│   └── reeb.py        - Reeb graph
├── neural_ode/    - Neural ODEs (torchdiffeq)
├── hawkes/        - Hawkes process cascades
├── inverse_rl/    - MaxEnt IRL (JAX)
├── lca/           - Carbon lifecycle analysis (JAX)
└── causal_graph/ - SURD decomposition
```

## Rules
- **No direct kernel calls** - all data via boundary/ffi
- **No Elixir/TS imports**
- Output goes back through boundary

## Dependencies
- gudhi, ripser - TDA
- jax, torch, torchdiffeq - ML
- ray - Distributed compute
# Database - TimescaleDB Schemas

## Migrations
```
db/
├── migrations/
│   ├── 001_simulation_trajectories.sql
│   ├── 002_invariant_audit_log.sql
│   ├── 003_city_system_state.sql
│   ├── 004_asset_degradation.sql
│   ├── 005_federated_model_registry.sql
│   └── 006_design_lifecycle.sql
├── hypertables/
│   └── compression_policies.sql
└── seeds/
    └── test_fixtures.sql
```

## Key Tables
- `simulation_trajectories` - Product, entity_id, state_vector (FlatBuffer), lyapunov_max
- `invariant_audit_log` - gate_result, confidence, violation_trace (JSONB), escalated
- `city_system_state` - district_id, system, chaos_index, verifiable
- `asset_degradation` - asset_id, sde_state, aleatoric_var, epistemic_var, regime_flag
- `federated_model_registry` - model_hash, commit_proof, ocaml_status
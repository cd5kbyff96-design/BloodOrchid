# Federation - Consensus + Quorum

## Structure
```
federation/
├── consensus/    - Rust: deterministic ordering + cryptographic signing
├── aggregation/  - Rust: secure gradient aggregation
├── quorum/       - Elixir: membership rotation + async batching
├── speculative/  - Elixir: optimistic execution
└── registry/     - Elixir: model versioning → TimescaleDB
```

## HotStuff BFT
- Rust consensus for deterministic ordering
- Elixir quorum for membership rotation

## Speculative Execution
Eligible when:
- OCaml gate returns Valid
- loss_delta < threshold
- No adversarial challenge in last N rounds

Otherwise → full pipelined BFT
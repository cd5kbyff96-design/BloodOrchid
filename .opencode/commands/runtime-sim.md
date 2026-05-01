# Runtime Simulation Command

## Usage
```bash
opencode runtime-sim [options]
```

## Purpose
Simulate the full runtime behavior without actually executing the simulation.

## Options
- `--steps N` - Simulate N steps (default: 100)
- `--seed S` - Use seed S for determinism
- `--output FILE` - Write trace to FILE

## What it validates
1. Data flow from kernel → boundary → causal → invariants
2. Serialization/deserialization roundtrips
3. Memory allocation patterns
4. SLO compliance (simulated timing)

## Example
```bash
opencode runtime-sim --steps 1000 --seed 42
```

## Output
JSON trace with timestamps, memory usage, and any violations detected.
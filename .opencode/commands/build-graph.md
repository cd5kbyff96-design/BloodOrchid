# Build Graph Command

## Usage
```bash
opencode build-graph
```

## What it shows
- Dependency graph of all crates
- Build order recommendations
- Circular dependency detection
- Missing dependencies

## Example
```
kernel (C++) → boundary (Rust) → causal (Python)
                    ↓
              invariants (OCaml)
```

## Output format
DOT graph for visualization:
```bash
opencode build-graph | dot -Tpng > graph.png
```
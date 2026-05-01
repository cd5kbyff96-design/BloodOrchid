# Audit Command

## Usage
```bash
opencode audit
```

## What it checks

### 1. Build Health
- `cargo build --workspace` succeeds
- No compilation warnings
- All tests pass

### 2. Dependency Graph
- No circular dependencies
- No unauthorized cross-language imports
- Contract coverage matrix is complete

### 3. Migration State
- Shadow directories are tracked
- Dual-write layer is stable
- No orphaned imports

## Output
Returns exit code 0 if all checks pass, non-zero otherwise.
Writes detailed report to stderr.

## Example
```bash
$ opencode audit
✓ Build: OK
✓ Dependencies: OK
✓ Migration: OK
Audit complete: PASSED
```
# OpenCode Agent Pack

## Role
You are an expert systems engineer helping with the Arrakis monorepo.

## Constraints
- Never break the build
- Always verify with `cargo test --workspace`
- Follow docs/architecture.md as source of truth
- Use `opencode audit` before any commit

## Available Commands
- `opencode audit` - Check build health, deps, migration state
- `opencode build-graph` - Show dependency graph
- `opencode readiness` - Production readiness checklist
- `opencode runtime-sim` - Simulate runtime behavior

## Workflow
1. Understand the task
2. Make minimal changes
3. Run tests
4. Commit with clear message
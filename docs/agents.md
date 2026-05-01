# AI Tooling Policy

## Supported Agents
- **OpenCode** - Primary CLI assistant
- **Cursor** - IDE integration with rules in `.cursor/rules/`
- **Zed** - Code editor with agent pack
- **Gemini** - Reasoning engine
- **Codex** - Low-level implementation

## Rules for All AI Systems
1. **NEVER commit automatically** - always present diff to user
2. **NEVER push to remote** - user must approve push
3. **MUST respect source of truth** - docs/architecture.md, docs/contracts.md
4. **MUST verify build** - run `cargo test --workspace` before any PR

## Agent Packs
Located in `docs/agent_packs/`:
- `opencode/` - OpenCode CLI commands
- `cursor/` - IDE rules (00-core, 01-boundaries, 02-build-order, 03-test)
- `zed/` - Refactoring guidelines
- `gemini/` - Cross-module reasoning
- `codex/` - Low-level C++/Rust implementation

## OpenCode Commands
```bash
opencode audit          # Check build health
opencode build-graph   # Show dependency graph  
opencode readiness    # Production readiness
opencode runtime-sim   # Simulate runtime behavior
```
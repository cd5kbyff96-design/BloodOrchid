Welcome — how to contribute

Branching
- Use feature/<short-description> or repo-reorg/<what> for reorg branches.

Commits
- Use Conventional Commits (type: scope): Short description
- Example: feat(rust/crates/core_sim): add physics core

PRs
- Keep PRs small (one package or one service at a time).
- Link to the task in the description and reference CODEBASE.md to explain layout changes.

Testing
- Run language-specific tests locally (cargo test, mix test, pytest)
- Run formatters and linters (see CODEBASE.md for tools)

Moving files
- Use git mv to move files to preserve history.
- After moving, update package manifests (Cargo.toml, mix.exs, pyproject.toml).
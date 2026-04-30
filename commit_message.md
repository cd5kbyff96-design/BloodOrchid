# Refactor Commit

This commit organizes the project structure for better maintainability and clarity. Below are the changes made:

- Centralized protocol definitions in `contracts/proto/`
- Organized tests into dedicated directories for unit, integration, and end-to-end tests.
- Cleaned up documentation to maintain clarity and avoid duplication.
- Created a dedicated directory for Bazel builds.

### Changes:

- **contracts/proto/** centralized (5 families)
- **tests/** reorganized into {unit, integration, e2e}
- **docs/** pure separation
- **bazel/** workspace central

Closes ARR-XXX
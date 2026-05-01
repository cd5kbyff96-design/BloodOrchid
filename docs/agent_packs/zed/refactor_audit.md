# Zed Agent Pack

## Role
You are assisting with refactoring operations in the Arrakis codebase.

## Refactoring Rules
1. **Never move + refactor in same commit** - separate into two steps
2. **Always verify tests pass** after any change
3. **Check for import regressions** - old paths must still work (dual-write layer)
4. **Update affected documentation** if contracts/ changes

## Before Refactoring
- Run `opencode audit` to understand current state
- Identify all call sites
- Plan migration path

## After Refactoring
- Run full test suite
- Check for dead imports
- Update Cursor rules if boundaries change
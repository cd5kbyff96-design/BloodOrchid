# Production Readiness Command

## Usage
```bash
opencode readiness
```

## Readiness Checklist

### Code Quality
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Documentation updated

### Dependencies
- [ ] No transitive deps on unverified crates
- [ ] Lockfile updated
- [ ] No security advisories

### Testing
- [ ] Unit test coverage > 80%
- [ ] Integration tests pass
- [ ] E2E smoke tests pass
- [ ] Performance baseline established

### Deployment
- [ ] Docker image builds
- [ ] Helm chart valid
- [ ] Terraform plan succeeds
- [ ] CI pipeline green

## Exit codes
- 0 = Ready for production
- 1 = Not ready (check output for details)
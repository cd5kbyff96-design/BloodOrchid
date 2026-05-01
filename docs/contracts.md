# Contracts

## Overview
All inter-module communication must go through contract definitions in `contracts/`.

## Contract Directories

### shared/
- `envelope.proto` - TraceContext, EndpointIdentity
- `errors.proto` - ErrorCode enum, ContractError
- `streaming.proto` - StreamWindow, StreamCursor
- `versioning.proto` - ContractVersion, SchemaEvolution

### kernel_boundary/
- `contracts.proto` - KernelStepRequest/Result, KernelStateVector

### boundary_ml/
- `contracts.proto` - FeatureBatch, InferenceRequest/Response

### boundary_invariants/
- `contracts.proto` - InvariantInput, InvariantViolation, EvaluationRequest/Result

### boundary_services/
- `contracts.proto` - PublishRequest/Receipt, SubscriptionRequest

### services_apps/
- `contracts.proto` - AppCommand, AppQuery, SessionProjection

### edge_cloud/
- `contracts.proto` - SyncRequest, SyncChunk, ReconciliationResult

## Rules
- No business logic in contracts/
- Every inter-module hop has exactly one contract
- Breaking changes = new message name + major version bump
- Supported formats: Proto, FlatBuffers

## Test Matrix
See `contracts/test_matrix.md` for mandatory CI test gates.
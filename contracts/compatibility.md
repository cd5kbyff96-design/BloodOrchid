# Contract Compatibility Policy

This document defines mandatory schema evolution rules for Veil Iris contracts.

- Scope: all files under `contracts/` and all generated artifacts derived from them.
- Enforcement: CI must fail on any rule violation in this document.
- Authority: this policy is normative for cross-module compatibility decisions.

## 1) Compatibility Model

### 1.1 Contract Families

A **contract family** is a stable message/interface namespace (for example, `veiliris.contracts.kernel_boundary`).

Every externally visible message or table must belong to exactly one contract family and have:

- `major` version (breaking boundary)
- `minor` version (backward-compatible feature additions)
- `patch` version (non-structural correction: docs/comments/options/constraints tightening only)

### 1.2 Transport Classification

- **Protocol Buffers (`proto3`)**: external/distributed contracts (boundary crossing, service federation, edge-cloud exchange).
- **FlatBuffers**: internal/high-frequency contracts (intra-runtime hot paths, low-latency binary layouts).

No module may invent ad hoc wire formats outside this classification.

## 2) Protocol Buffers Rules (External/Distributed)

## 2.1 Allowed Changes (Same Major)

The following are allowed with `minor` increment:

1. Add a new field with a new field number.
2. Add a new enum value at the end.
3. Add a new message type.
4. Add a new RPC/service method only if existing methods are unchanged.
5. Mark old fields as deprecated (`[deprecated = true]`) while retaining tag and wire compatibility.
6. Reserve removed field numbers and names immediately after removal.

The following are allowed with `patch` increment:

1. Clarify comments/docs.
2. Add stricter validation rules in documentation only when old payloads remain wire-compatible.
3. Adjust codegen options that do not alter wire format.

## 2.2 Disallowed Changes (Breaking)

The following are prohibited within the same `major`:

1. Renumber an existing field tag.
2. Reuse a previously used field tag.
3. Change field wire type (for example `int32` -> `string`, `bytes` -> `message`).
4. Change cardinality (`optional/singular/repeated`) in a non-wire-compatible way.
5. Move an existing field into or out of a `oneof`.
6. Delete enum values and reuse numeric values.
7. Rename package/family namespace without major migration.
8. Remove an RPC/service method still declared supported.

Any disallowed change requires:

- new message/service symbol names,
- `major` increment,
- migration notes and dual-read/dual-write plan where applicable.

## 2.3 Backward vs Forward Compatibility

- **Backward compatibility required** for all `minor`/`patch` releases:
  - New consumers must parse data from previous versions.
- **Forward compatibility required** for all `minor` releases:
  - Old consumers must ignore unknown fields and continue operating safely.
- **No forward guarantee across major boundaries** unless explicitly stated by a migration profile.

Rule: distributed producers must not emit new required semantics that old consumers cannot safely ignore within same major.

## 2.4 Deprecation Lifecycle (Protocol Buffers)

Deprecation is mandatory and time-bounded:

1. **Phase D1 (Introduce Replacement)**:
   - Add replacement field/message.
   - Keep old field active.
   - Increment `minor`.
2. **Phase D2 (Deprecate)**:
   - Mark old symbol deprecated.
   - Add explicit migration notes in contract changelog.
   - Keep wire compatibility.
3. **Phase D3 (Removal Window)**:
   - After at least 2 release cycles and after telemetry confirms zero usage, remove old symbol.
   - Reserve field number and name.
   - Increment `major`.

CI must block removal if usage evidence or migration note is missing.

## 2.5 Proto Good vs Bad Examples

### Good (Additive, Minor)

```proto
message InferenceRequest {
  string request_id = 1;
  bytes features = 2;
  string model_id = 3; // added in 1.4.0
}
```

### Bad (Tag Reuse, Breaking)

```proto
message InferenceRequest {
  string request_id = 1;
  string model_id = 2; // BAD: tag 2 was previously bytes features
}
```

### Good (Deprecate with Replacement)

```proto
message PublishReceipt {
  string broker_id = 4;
  string broker_instance_id = 6; // replacement
  string broker_id_legacy = 7 [deprecated = true];
}
```

### Bad (Delete Without Reserve)

```proto
message PublishReceipt {
  string broker_id = 4; // removed later without reserving 4 -> disallowed
}
```

## 3) FlatBuffers Rules (Internal/High-Frequency)

## 3.1 Allowed Changes (Same Major)

Allowed with `minor` increment:

1. Add new table fields with new ids and defaults.
2. Add new tables/structs/enums.
3. Add enum values only at the end (no numeric reuse).
4. Add unions members only when all readers handle unknown variants safely.

Allowed with `patch` increment:

1. Non-structural docs/comments updates.
2. Generated-code-only option changes that do not alter serialized layout.

## 3.2 Disallowed Changes (Breaking)

Prohibited within same major:

1. Reorder or renumber fields in a way that changes semantic mapping.
2. Change scalar widths (`int32` -> `int64`) for existing field ids.
3. Convert `table` to `struct` or `struct` to `table`.
4. Remove fields without compatibility adapter and major bump.
5. Reuse enum numeric ids.
6. Change union member numeric mapping.

Any disallowed change requires a new major schema and migration translator.

## 3.3 Backward vs Forward Compatibility

- Readers must tolerate absent newer fields and use defaults.
- Writers must not rely on newly added fields being interpreted by older readers.
- For high-frequency paths, unknown/unsupported union variants must fail closed with explicit error metrics, not undefined behavior.

## 3.4 Deprecation Lifecycle (FlatBuffers)

1. Introduce replacement field/table (`minor`).
2. Mark legacy field deprecated in schema comments and compatibility manifest.
3. Keep dual-read support for at least 2 internal releases.
4. Remove only in next `major`, with converter retained for rollback window.

CI must require converter tests before major removals.

## 3.5 FlatBuffers Good vs Bad Examples

### Good (Append Field)

```fbs
table KernelStateVector {
  state_id:string (id: 0);
  tick:ulong (id: 1);
  serialized_state:[ubyte] (id: 2);
  compression:string (id: 3); // added in minor release
}
```

### Bad (Type Mutation on Existing id)

```fbs
table KernelStateVector {
  state_id:string (id: 0);
  tick:string (id: 1); // BAD: was ulong, same id now different type
}
```

### Good (Major Migration)

```fbs
// v2 major:
table KernelStateVectorV2 {
  state_id:string (id: 0);
  tick:ulong (id: 1);
  serialized_state:[ubyte] (id: 2);
  checksum:string (id: 3);
}
```

## 4) Versioning Strategy

## 4.1 Semantic Versioning Rules

- `major`: any breaking structural or semantic change.
- `minor`: additive compatible changes only.
- `patch`: non-structural updates only.

Each contract family must publish:

1. current version,
2. compatibility guarantee (`BACKWARD`, `FORWARD`, `FULL`, `BREAKING`),
3. deprecated symbols list,
4. migration notes.

## 4.2 Cross-Family Coordination

No dependent module may adopt a new contract `major` until:

1. upstream family is merged,
2. boundary adapters are merged,
3. contract tests pass in both producer and consumer pipelines.

Skipping layers is forbidden; `kernel` cannot bypass `boundary`, `apps` cannot bypass `services`, etc.

## 5) Cross-Language Safety Rules

All generated bindings for Rust, C++, Python, OCaml, and Elixir must satisfy these constraints:

1. **Numeric safety**:
   - No implicit narrowing conversions across language boundaries.
   - 64-bit numeric fields must have explicit handling in Python and Elixir wrappers.
2. **Enum safety**:
   - Unknown enum values must map to `UNSPECIFIED` or explicit unknown branch; never crash by default.
3. **Presence semantics**:
   - Optional/presence-sensitive fields must not be conflated with zero-values.
4. **Bytes and ownership**:
   - No unsafe aliasing assumptions between Rust/C++ memory and managed runtimes.
5. **Determinism**:
   - Map iteration order must never be used for deterministic logic unless explicitly sorted.
6. **Error normalization**:
   - All boundary errors must map to `ContractError` envelope.
7. **Time semantics**:
   - Timestamps must be UTC and serialized as contract-defined types only.

Language-specific mandatory checks:

- **Rust**: deny unchecked `as` casts in generated adapter layers; enforce `TryFrom` for narrowing.
- **C++**: compile with warnings-as-errors for conversion/sign warnings in contract adapters.
- **Python**: validate integer ranges before serialization.
- **OCaml**: exhaustive pattern matches for contract enums/unions.
- **Elixir**: explicit decoder fallback for unknown fields/enum values.

CI must execute cross-language round-trip tests for all contract families touched by a change.

## 6) CI Enforcement Rules

All compatibility checks are mandatory in CI:

1. **Schema diff gate**:
   - Detect breaking changes using previous baseline.
2. **Reserved tag gate (Proto)**:
   - Block field deletion without `reserved`.
3. **FlatBuffers layout gate**:
   - Block id/type mutations without major bump.
4. **Version gate**:
   - Block incompatible schema diffs without correct `major` increment.
5. **Deprecation gate**:
   - Block removal without documented lifecycle completion.
6. **Cross-language codegen gate**:
   - Generate and compile bindings for Rust/C++/Python/OCaml/Elixir adapters.
7. **Round-trip gate**:
   - Encode/decode compatibility tests across N-1 and N versions.

Merge is blocked if any gate fails.

## 7) Required Artifacts Per Contract Change

Every contract PR must include:

1. Updated schema files.
2. Updated version metadata in envelope/version declarations.
3. Changelog entry with compatibility classification.
4. Added/updated compatibility tests.
5. Migration plan for any deprecated or breaking symbol.

Changes missing any artifact are non-compliant and must be rejected.

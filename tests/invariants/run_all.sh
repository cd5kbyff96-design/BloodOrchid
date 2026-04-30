#!/bin/sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)"

PROTO_PATH="$REPO_ROOT/contracts/boundary_invariants/contracts.proto"
MLI_PATH="$REPO_ROOT/invariants/ocaml/invariant_gate.mli"
ML_PATH="$REPO_ROOT/invariants/ocaml/invariant_gate.ml"

[ -f "$PROTO_PATH" ]
[ -f "$MLI_PATH" ]
[ -f "$ML_PATH" ]

# Ensure the OCaml stub explicitly aligns with the expected boundary invariant contract shape.
grep -q "message InvariantRequest" "$PROTO_PATH"
grep -q "message InvariantResponse" "$PROTO_PATH"
grep -q "simulation_state" "$PROTO_PATH"
grep -q "accepted" "$PROTO_PATH"
grep -q "violations" "$PROTO_PATH"

grep -q "type request" "$MLI_PATH"
grep -q "type response" "$MLI_PATH"
grep -q "val evaluate" "$MLI_PATH"
grep -q "let evaluate" "$ML_PATH"

echo "invariant_gate_tests: PASS"

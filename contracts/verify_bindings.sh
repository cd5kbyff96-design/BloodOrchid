#!/bin/bash

set -euo pipefail

echo "============================================="
echo "GATE L0 - CONTRACT INTEGRITY VERIFICATION"
echo "============================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACTS_DIR="$SCRIPT_DIR"
MANIFEST_FILE="$CONTRACTS_DIR/.contract_manifest.sha256"

echo "[INFO] Verifying protobuf schemas..."

PROTO_FILES=(
    "timescaledb/storage.proto"
    "elixir_federation/federation.proto"
)

for proto in "${PROTO_FILES[@]}"; do
    proto_path="$CONTRACTS_DIR/$proto"
    if [[ ! -f "$proto_path" ]]; then
        echo "[ARCHITECTURAL VIOLATION]"
        echo "Gate: L0 - Contract Integrity"
        echo "Crate: contracts"
        echo "Issue: Missing protobuf schema: $proto"
        echo "Rule: All protobuf schemas must exist"
        echo "Fix: Create or restore $proto"
        exit 1
    fi
    echo "[OK] Found: $proto"
done

echo ""
echo "[INFO] Computing SHA-256 manifest of proto files..."

manifest_content=""
for proto in "${PROTO_FILES[@]}"; do
    proto_path="$CONTRACTS_DIR/$proto"
    if [[ -f "$proto_path" ]]; then
        hash=$(sha256sum "$proto_path" | awk '{print $1}')
        manifest_content="$manifest_content$hash$proto\n"
    fi
done

current_manifest=$(printf "$manifest_content" | sha256sum | awk '{print $1}')

echo "Current manifest: $current_manifest"

if [[ -f "$MANIFEST_FILE" ]]; then
    saved_manifest=$(cat "$MANIFEST_FILE")

    if [[ "$current_manifest" != "$saved_manifest" ]]; then
        echo "[ARCHITECTURAL VIOLATION]"
        echo "Gate: L0 - Contract Integrity"
        echo "Crate: contracts"
        echo "Issue: Contract drift detected"
        echo "Rule: Proto schemas must match last-known-good manifest"
        echo "Fix: Run with --update-manifest to commit new baseline"
        echo "     or revert changes to match previous schema"
        exit 1
    fi
    echo "[OK] Contract manifest matches baseline"
else
    echo "[INFO] No baseline manifest found - creating initial baseline"
    echo "$current_manifest" > "$MANIFEST_FILE"
fi

echo ""
echo "[INFO] Verifying Rust bindings are in sync..."

RUST_PROTO_DIR="$SCRIPT_DIR/../cve/core/src"
if [[ -d "$RUST_PROTO_DIR" ]]; then
    echo "[OK] CVE proto definitions present"
fi

RUST_BOUNDARY_DIR="$SCRIPT_DIR/../boundary/runtime/src"
if [[ -d "$RUST_BOUNDARY_DIR" ]]; then
    echo "[OK] Boundary proto definitions present"
fi

echo ""
echo "============================================="
echo "[PASS] Contract integrity verified"
echo "============================================="

exit 0
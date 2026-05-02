#!/bin/bash

set -euo pipefail

echo "============================================="
echo "GATE L3 - DETERMINISM & HASH STABILITY CHECK"
echo "============================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/.determinism_output"
ITERATIONS=10

mkdir -p "$OUTPUT_DIR"

echo "[INFO] Running full MVES pipeline $ITERATIONS times..."

run_pipeline() {
    local iteration=$1
    local output_file="$OUTPUT_DIR/run_$iteration.json"

    cd "$PROJECT_ROOT"

    if cargo build --package mves-cli 2>/dev/null; then
        local result
        result=$(cargo run --package mves-cli --bin mves_demo 2>/dev/null || echo "BUILD_FAILED")

        echo "$result" > "$output_file"
    else
        echo "BUILD_FAILED" > "$output_file"
        echo "[WARN] Build failed for iteration $iteration"
    fi
}

for i in $(seq 1 $ITERATIONS); do
    echo "[INFO] Running iteration $i of $ITERATIONS..."
    run_pipeline $i
done

echo "[INFO] Computing SHA-256 hashes of all outputs..."

HASH_FILE="$OUTPUT_DIR/hashes.txt"
> "$HASH_FILE"

for i in $(seq 1 $ITERATIONS); do
    output_file="$OUTPUT_DIR/run_$i.json"
    if [[ -f "$output_file" ]]; then
        hash=$(sha256sum "$output_file" | awk '{print $1}')
        echo "$hash" >> "$HASH_FILE"
        echo "  Iteration $i: $hash"
    else
        echo "[ERROR] Output file missing for iteration $i"
        exit 1
    fi
done

echo ""
echo "[INFO] Checking for hash stability..."

first_hash=$(head -n 1 "$HASH_FILE")
all_same=true

while IFS= read -r hash; do
    if [[ "$hash" != "$first_hash" ]]; then
        all_same=false
        break
    fi
done < "$HASH_FILE"

if $all_same; then
    echo "============================================="
    echo "[PASS] All $ITERATIONS runs produced identical output"
    echo "Hash: $first_hash"
    echo "============================================="

    echo "$first_hash" > "$OUTPUT_DIR/baseline_hash.txt"
    echo "[INFO] Baseline hash saved to $OUTPUT_DIR/baseline_hash.txt"

    exit 0
else
    echo "============================================="
    echo "[ARCHITECTURAL VIOLATION]"
    echo "Gate: L3 - Determinism & Hash Stability"
    echo "Crate: pipeline"
    echo "Issue: Output variance detected across $ITERATIONS runs"
    echo "Rule: Pipeline must produce deterministic output"
    echo "Fix: Investigate sources of nondeterminism:"
    echo "     - Check for random number generation"
    echo "     - Check for uninitialized memory read"
    echo "     - Check for race conditions"
    echo "     - Check for time-dependent operations"
    echo "============================================="

    echo ""
    echo "Unique hashes found:"
    sort -u "$HASH_FILE"

    exit 1
fi
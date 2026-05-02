#!/bin/bash

set -euo pipefail

echo "============================================="
echo "BLOODORCHID CI PIPELINE - 5 GATE ENFORCEMENT"
echo "============================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"

export PROJECT_ROOT
export CARGO_MANIFEST_DIR="$PROJECT_ROOT"

FAILED_GATE=""
FAILURE_DETAILS=""

run_gate() {
    local gate_num=$1
    local gate_name=$2
    local gate_script=$3

    echo ""
    echo "-------------------------------------------"
    echo "GATE $gate_num: $gate_name"
    echo "-------------------------------------------"

    if [[ -x "$SCRIPT_DIR/$gate_script" ]]; then
        if "$SCRIPT_DIR/$gate_script"; then
            echo "[GATE $gate_num] PASSED"
            return 0
        else
            FAILED_GATE="$gate_num"
            FAILURE_DETAILS="$gate_name"
            return 1
        fi
    elif [[ -x "$CONTRACTS_DIR/$gate_script" ]]; then
        if "$CONTRACTS_DIR/$gate_script"; then
            echo "[GATE $gate_num] PASSED"
            return 0
        else
            FAILED_GATE="$gate_num"
            FAILURE_DETAILS="$gate_name"
            return 1
        fi
    else
        echo "[GATE $gate_num] SKIPPED (script not found or not executable)"
        return 0
    fi
}

run_tests() {
    local test_name=$1

    echo ""
    echo "-------------------------------------------"
    echo "GATE L4: RUNNING $test_name"
    echo "-------------------------------------------"

    cd "$PROJECT_ROOT"

    if cargo test --package mves-cli --test architecture_test 2>/dev/null; then
        echo "[TEST] PASSED"
        return 0
    else
        echo "[ARCHITECTURAL VIOLATION]"
        echo "Gate: L4 - Architecture Rule Tests"
        echo "Issue: Architecture test suite failed"
        echo "Rule: All architecture tests must pass"
        echo "Fix: Review and fix failing architecture tests"
        return 1
    fi
}

echo "[INFO] Starting CI Pipeline..."

run_gate "L0" "Contract Integrity" "verify_bindings.sh" || { echo "[GATE L0] FAILED"; exit 1; }

run_gate "L1" "Layering & Dependency Guard" "arch_lint.sh" || { echo "[GATE L1] FAILED"; exit 1; }

run_gate "L2" "Purity & Statelessness Scan" "arch_lint.sh" || { echo "[GATE L2] FAILED"; exit 1; }

run_gate "L3" "Determinism & Hash Stability" "check_determinism.sh" || { echo "[GATE L3] FAILED"; exit 1; }

run_tests "architecture_test" || { echo "[GATE L4] FAILED"; exit 1; }

echo ""
echo "============================================="
echo "[SUCCESS] All CI gates passed!"
echo "============================================="

exit 0
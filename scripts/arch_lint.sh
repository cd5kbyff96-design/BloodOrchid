#!/bin/bash

set -euo pipefail

echo "============================================="
echo "ARCH-LINT: Architecture Enforcement Scanner"
echo "============================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

FAILED=0

scan_crate_for_violation() {
    local crate_path=$1
    local crate_name=$2
    local forbidden_pattern=$3
    local description=$4

    if [[ -d "$crate_path/src" ]]; then
        local violations=$(grep -r "$forbidden_pattern" "$crate_path/src" 2>/dev/null || true)

        if [[ -n "$violations" ]]; then
            echo "[ARCHITECTURAL VIOLATION]"
            echo "Gate: L2 - Purity & Statelessness Scan"
            echo "Crate: $crate_name"
            echo "Issue: Forbidden pattern detected: $description"
            echo "Rule: $crate_name must be fully hermetic"
            echo "Fix: Remove $forbidden_pattern from $crate_path/src"
            echo ""
            echo "Found at:"
            echo "$violations"
            echo "============================================="
            FAILED=1
        fi
    fi
}

echo "[L2] Scanning for state leakage and IO dependencies..."

CVE_DIR="$PROJECT_ROOT/cve/core"
scan_crate_for_violation "$CVE_DIR" "cve-core" "static mut" "mutable static variable"
scan_crate_for_violation "$CVE_DIR" "cve-core" "Mutex" "interior mutability"
scan_crate_for_violation "$CVE_DIR" "cve-core" "RefCell" "interior mutability"
scan_crate_for_violation "$CVE_DIR" "cve-core" "std::fs" "filesystem IO"
scan_crate_for_violation "$CVE_DIR" "cve-core" "std::net" "network IO"
scan_crate_for_violation "$CVE_DIR" "cve-core" "std::io" "IO dependency"
scan_crate_for_violation "$CVE_DIR" "cve-core" "lazy_static" "global state"
scan_crate_for_violation "$CVE_DIR" "cve-core" "once_cell" "global state"

echo "[L4] Scanning for kernel bypass..."

CLI_DIR="$PROJECT_ROOT/apps/cli"
scan_crate_for_violation "$CLI_DIR" "mves-cli" "KernelBridge" "direct kernel access"
scan_crate_for_violation "$CLI_DIR" "mves-cli" "kernel::" "kernel module access"

echo "[L1] Scanning for forbidden dependency edges..."

CVE_TOML="$CVE_DIR/Cargo.toml"
if [[ -f "$CVE_TOML" ]]; then
    if grep -q "kernel" "$CVE_TOML" 2>/dev/null; then
        echo "[ARCHITECTURAL VIOLATION]"
        echo "Gate: L1 - Layering Guard"
        echo "Crate: cve-core"
        echo "Issue: Forbidden dependency on kernel"
        echo "Rule: CVE must not depend on kernel"
        echo "Fix: Remove kernel dependency from cve/core/Cargo.toml"
        FAILED=1
    fi

    if grep -q "pde_ref" "$CVE_TOML" 2>/dev/null; then
        echo "[ARCHITECTURAL VIOLATION]"
        echo "Gate: L1 - Layering Guard"
        echo "Crate: cve-core"
        echo "Issue: Forbidden dependency on pde_ref"
        echo "Rule: CVE must not depend on kernel internals"
        echo "Fix: Remove pde_ref from cve/core/Cargo.toml"
        FAILED=1
    fi
fi

echo "[L4] Scanning for boundary bypass..."

if grep -r "KernelBridge" "$CLI_DIR/src" 2>/dev/null | grep -v "boundary" >/dev/null; then
    echo "[ARCHITECTURAL VIOLATION]"
    echo "Gate: L4 - Architecture Rule Tests"
    echo "Crate: mves-cli"
    echo "Issue: Boundary bypass detected"
    echo "Rule: CLI must route through Boundary, not access Kernel directly"
    echo "Fix: Remove direct KernelBridge usage from CLI code"
    FAILED=1
fi

if [[ $FAILED -eq 1 ]]; then
    echo ""
    echo "============================================="
    echo "[FAIL] Architecture violations detected"
    echo "============================================="
    exit 1
fi

echo ""
echo "============================================="
echo "[PASS] No architecture violations detected"
echo "============================================="

exit 0
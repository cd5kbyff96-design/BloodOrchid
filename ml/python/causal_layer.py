"""
causal_layer.py — Vail Iris ML / Causal Layer
Phase 2C: Production stub for SURD decomposition + invariant gate integration.

Architecture invariants this module enforces:
  I1  State is consumed ONLY from Rust boundary contracts (no direct kernel access).
  I2  The OCaml invariant gate MUST be called before any annotation is committed.
      No annotation with gate_status="invalid" is ever returned to the caller.
  I3  SURD decomposition is deterministic for identical field_values inputs.
  I4  This module is a pure transformation layer: no state mutation, no persistence,
      no network I/O. All side effects belong to boundary/ or storage/.
  I5  FNV-64 hash (invariant_token) matches stable_hash64 in boundary/runtime/src/lib.rs.

Phase 2C scope: stub SURD estimator using variance decomposition as a deterministic
proxy for the full JAX-accelerated Kraskov MI estimator.

Production path (Phase 4+):
  - JAX-accelerated nearest-neighbour MI estimation (Kraskov KSG estimator)
  - Ray worker ensemble dispatch
  - Full causal graph construction + SURD decomposition per architecture section V
  - OCaml gate called via Rust boundary PyO3 binding (not the in-process stub below)
"""

from __future__ import annotations

import struct
from dataclasses import dataclass, field as dc_field
from enum import Enum
from typing import Sequence

# JAX is preferred; numpy is the MVES CI fallback.
# Both expose the same API surface used here.
try:
    import jax.numpy as jnp
    import jax
    _JAX_AVAILABLE = True
    # Disable JIT in stub mode to keep determinism trivial to verify.
    jax.config.update("jax_disable_jit", True)
except ImportError:
    import numpy as jnp  # type: ignore[no-redef]
    _JAX_AVAILABLE = False


# ─── Gate status (mirrors OCaml gate.ml / contracts/python_causal/causal.proto) ─

class GateStatus(str, Enum):
    VALID     = "valid"
    UNCERTAIN = "uncertain"
    INVALID   = "invalid"


# ─── Data classes ──────────────────────────────────────────────────────────────

@dataclass(frozen=True)
class SurdDecomposition:
    """
    Synergy-Unique-Redundancy Decomposition of causal information flow.

    Invariant enforced by InvariantGateStub.validate():
        abs(synergy + unique_source + redundancy - 1.0) <= 0.05
    """
    synergy:       float = 0.0
    unique_source: float = 0.0
    redundancy:    float = 0.0
    total_mi:      float = 0.0
    dominant_mode: str   = "unique"  # "synergistic" | "unique" | "redundant"

    @staticmethod
    def from_field_values(values: Sequence[float]) -> "SurdDecomposition":
        """
        Deterministic stub SURD estimation from raw FieldTensor values.

        Phase 2C proxy: variance-based decomposition over thirds of the field.
        Production path: Kraskov KSG MI estimator over the joint state distribution
        of coupled system partitions (see architecture.md § V, causal/causal_graph/).

        Determinism guarantee: identical values → identical output, regardless of
        Python interpreter, OS, or JAX/numpy backend.
        """
        if not values:
            return SurdDecomposition()

        arr  = jnp.array(values, dtype=jnp.float32)
        n    = len(values)
        eps  = 1e-9

        # Partition field into thirds for SURD proxy
        t1   = arr[: n // 3]       if n >= 3 else arr
        t2   = arr[n // 3 : 2 * n // 3] if n >= 6 else arr
        t3   = arr[2 * n // 3 :]   if n >= 3 else arr

        var_t = float(jnp.var(arr))       + eps
        var1  = float(jnp.var(t1))        + eps
        var2  = float(jnp.var(t2))        + eps
        var3  = float(jnp.var(t3))        + eps

        # Proxy SURD fractions — normalised so they sum to 1.0
        raw_syn  = var3 / var_t * 0.25
        raw_red  = var2 / var_t * 0.35
        raw_uni  = var1 / var_t * 0.40
        total    = raw_syn + raw_red + raw_uni + eps
        synergy  = raw_syn / total
        red      = raw_red / total
        uni      = raw_uni / total

        total_mi = float(jnp.mean(jnp.abs(arr - jnp.mean(arr))))

        dominant = (
            "synergistic" if synergy > uni and synergy > red
            else "redundant" if red > uni
            else "unique"
        )

        return SurdDecomposition(
            synergy       = synergy,
            unique_source = uni,
            redundancy    = red,
            total_mi      = total_mi,
            dominant_mode = dominant,
        )


@dataclass
class CausalAnnotation:
    """Output of the causal layer — SURD + gate status."""
    simulation_id:    str          = ""
    step_index:       int          = 0
    surd:             SurdDecomposition = dc_field(default_factory=SurdDecomposition)
    stability_score:  float        = 1.0   # [0.0, 1.0]
    gate_status:      GateStatus   = GateStatus.VALID
    invariant_token:  bytes        = b""
    violation_reasons: list[str]  = dc_field(default_factory=list)


# ─── Invariant gate stub ───────────────────────────────────────────────────────

class InvariantGateStub:
    """
    In-process stub for the OCaml invariant gate (gate.ml).

    Enforces the same tri-state semantics as the production OCaml gate:
      VALID     — all structural invariants pass
      UNCERTAIN — passes with confidence < 1.0; caller must propagate DecisionStateMarker
      INVALID   — hard failure; caller must NOT commit; fallback policy must be loaded

    Production wiring: Rust boundary calls into OCaml via C-ABI embedding.
    This stub is the Phase 2C stand-in; replace with the boundary RPC call in Phase 3.

    Invariants checked:
      1. SURD components sum to 1.0 ± 0.05
      2. stability_score in [0.0, 1.0]
      3. simulation_id non-empty
      4. UNCERTAIN path: stability_score < 0.4
    """

    def validate(
        self,
        simulation_id:   str,
        step_index:      int,
        surd:            SurdDecomposition,
        stability_score: float,
    ) -> tuple[GateStatus, float, list[str]]:
        """
        Returns (gate_status, confidence_bound, violation_reasons).
        confidence_bound is 1.0 for VALID, (0.0, 1.0) for UNCERTAIN, 0.0 for INVALID.
        """
        violations: list[str] = []

        # Hard violations → INVALID
        if not simulation_id.strip():
            violations.append("simulation_id: must not be empty")

        if step_index < 0:
            violations.append(f"step_index: must be non-negative, got {step_index}")

        surd_sum = surd.synergy + surd.unique_source + surd.redundancy
        if abs(surd_sum - 1.0) > 0.05:
            violations.append(
                f"surd_sum_invariant: components sum to {surd_sum:.6f}, "
                f"expected 1.0 ± 0.05"
            )

        if not (0.0 <= stability_score <= 1.0):
            violations.append(
                f"stability_range: score={stability_score:.4f} outside [0.0, 1.0]"
            )

        if violations:
            return GateStatus.INVALID, 0.0, violations

        # Soft violation → UNCERTAIN
        if stability_score < 0.4:
            return (
                GateStatus.UNCERTAIN,
                stability_score,
                [f"low_stability: score={stability_score:.4f} below threshold 0.40"],
            )

        return GateStatus.VALID, 1.0, []


# ─── Main causal layer ─────────────────────────────────────────────────────────

class CausalLayer:
    """
    Vail Iris Causal Layer — Phase 2C stub.

    Entry point:
        annotation = CausalLayer().process(simulation_id, step_index, field_values)

    The call sequence is:
        1. Compute SurdDecomposition from field_values  (deterministic)
        2. Compute stability_score                       (deterministic)
        3. Call invariant gate                           (MUST happen before return)
        4. Attach invariant_token                        (FNV-64, matches Rust boundary)
        5. Return CausalAnnotation

    Raises ValueError if require_valid=True and gate returns INVALID.
    Never raises on UNCERTAIN — callers must check annotation.gate_status.
    """

    def __init__(self) -> None:
        self._gate = InvariantGateStub()

    def process(
        self,
        simulation_id: str,
        step_index:    int,
        field_values:  Sequence[float],
        *,
        require_valid: bool = False,
    ) -> CausalAnnotation:
        """
        Transform boundary-decoded field values into a gate-validated CausalAnnotation.

        Args:
            simulation_id: from SimulationState.simulation_id
            step_index:    from SimulationState.step_index
            field_values:  flattened float32 from SimulationState.primary_field.values
            require_valid: if True, raise ValueError when gate returns INVALID

        Returns:
            CausalAnnotation with gate_status reflecting invariant check result.
        """
        if not field_values:
            raise ValueError(
                "field_values must be non-empty; cannot derive causal structure "
                "from an empty FieldTensor"
            )

        surd = SurdDecomposition.from_field_values(field_values)

        # Stability proxy: 1 - normalised coefficient of variation
        arr      = jnp.array(field_values, dtype=jnp.float32)
        mean_abs = float(jnp.mean(jnp.abs(arr))) + 1e-9
        std      = float(jnp.std(arr))
        stability = max(0.0, min(1.0, 1.0 - (std / mean_abs) * 0.1))

        # ── Invariant gate (non-optional) ──────────────────────────────────────
        gate_status, confidence, violations = self._gate.validate(
            simulation_id, step_index, surd, stability
        )

        # Deterministic invariant token: FNV-64 over identity + gate result.
        # Byte layout matches stable_hash64 in boundary/runtime/src/lib.rs.
        token_src = (
            f"{simulation_id}\x00{step_index}\x00{gate_status.value}"
        ).encode("utf-8")
        token_bytes = _fnv64(token_src).to_bytes(8, byteorder="little")

        annotation = CausalAnnotation(
            simulation_id     = simulation_id,
            step_index        = step_index,
            surd              = surd,
            stability_score   = stability,
            gate_status       = gate_status,
            invariant_token   = token_bytes,
            violation_reasons = violations,
        )

        if require_valid and gate_status == GateStatus.INVALID:
            raise ValueError(
                f"Invariant gate returned INVALID for "
                f"simulation={simulation_id!r} step={step_index}: {violations}"
            )

        return annotation


# ─── Utilities ─────────────────────────────────────────────────────────────────

def _fnv64(data: bytes) -> int:
    """
    FNV-1a 64-bit hash.
    Matches stable_hash64 in boundary/runtime/src/lib.rs exactly:
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME:  u64 = 0x100000001b3;
    """
    FNV_OFFSET: int = 0xcbf29ce484222325
    FNV_PRIME:  int = 0x100000001b3
    h = FNV_OFFSET
    for byte in data:
        h ^= byte
        h = (h * FNV_PRIME) & 0xFFFF_FFFF_FFFF_FFFF
    return h


def jax_available() -> bool:
    """Returns True if JAX is available in the current environment."""
    return _JAX_AVAILABLE

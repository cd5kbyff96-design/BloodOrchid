-- storage/timescaledb/schema.sql
-- Vail Iris — TimescaleDB Schema Reference
-- Phase 2C: MVES-aligned hypertables.
--
-- This file is the human-readable reference schema.
-- The authoritative migration is migrations/001_init.up.sql.
-- Both must remain identical in DDL structure.
--
-- Architecture rules:
--   - All tables are append-only write paths (no UPDATE on committed rows)
--   - simulation_id + step_index is the canonical identity key
--   - state_vector stores raw protobuf bytes (serialized SimulationState)
--   - state_hash is the 16-char FNV-64 hex from boundary/runtime::stable_hash64
--   - invariant_audit_log rows are immutable once inserted
--   - Only the Rust boundary layer writes; Python/Elixir read only
--
-- Requires: TimescaleDB extension >= 2.13, PostgreSQL >= 15

-- ─── Extensions ──────────────────────────────────────────────────────────────

CREATE EXTENSION IF NOT EXISTS timescaledb;

-- ─── simulation_trajectories ─────────────────────────────────────────────────
-- One row per (simulation_id, step_index) output from the MVES pipeline.
-- state_vector is the raw boundary-validated SimulationState protobuf bytes.
-- state_hash must match stable_hash64(state_vector) from boundary/runtime.

CREATE TABLE IF NOT EXISTS simulation_trajectories (
    time             TIMESTAMPTZ      NOT NULL,
    product          TEXT             NOT NULL DEFAULT 'mves',
    simulation_id    TEXT             NOT NULL,
    step_index       BIGINT           NOT NULL,
    state_vector     BYTEA            NOT NULL,
    state_hash       TEXT             NOT NULL,
    solver_kind      TEXT             NOT NULL,
    simulation_time  DOUBLE PRECISION NOT NULL,
    regime           TEXT             NOT NULL DEFAULT 'unknown',

    CONSTRAINT traj_hash_nonempty    CHECK (length(state_hash) = 16),
    CONSTRAINT traj_step_nonneg      CHECK (step_index >= 0),
    CONSTRAINT traj_simtime_nonneg   CHECK (simulation_time >= 0.0),
    CONSTRAINT traj_product_valid    CHECK (product IN ('mves', 'studio', 'hyperlattice', 'echoinfra')),
    CONSTRAINT traj_regime_valid     CHECK (regime  IN ('stable', 'transient', 'chaotic', 'unknown'))
);

SELECT create_hypertable(
    'simulation_trajectories',
    'time',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists       => TRUE
);

CREATE INDEX IF NOT EXISTS idx_traj_sim_step
    ON simulation_trajectories (simulation_id, step_index DESC);

CREATE INDEX IF NOT EXISTS idx_traj_product_time
    ON simulation_trajectories (product, time DESC);

-- ─── invariant_audit_log ─────────────────────────────────────────────────────
-- Immutable audit log for every OCaml gate decision in the MVES pipeline.
-- gate_result drives downstream behaviour:
--   "valid"     → annotation committed, forwarded normally
--   "uncertain" → annotation committed with DecisionStateMarker propagated
--   "invalid"   → annotation rejected; fallback_policy_id loaded; escalated → true

CREATE TABLE IF NOT EXISTS invariant_audit_log (
    time               TIMESTAMPTZ NOT NULL,
    pipeline           TEXT        NOT NULL,
    simulation_id      TEXT        NOT NULL,
    step_index         BIGINT      NOT NULL,
    gate_result        TEXT        NOT NULL,
    confidence         REAL,
    violation_trace    TEXT,        -- JSON array of violation_reason strings
    fallback_policy_id TEXT,        -- non-empty iff gate_result = 'invalid'
    escalated          BOOLEAN     NOT NULL DEFAULT FALSE,

    CONSTRAINT audit_gate_valid    CHECK (gate_result IN ('valid', 'uncertain', 'invalid')),
    CONSTRAINT audit_step_nonneg   CHECK (step_index >= 0),
    CONSTRAINT audit_confidence_rng CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0))
);

SELECT create_hypertable(
    'invariant_audit_log',
    'time',
    if_not_exists => TRUE
);

CREATE INDEX IF NOT EXISTS idx_audit_sim_gate
    ON invariant_audit_log (simulation_id, gate_result, time DESC);

CREATE INDEX IF NOT EXISTS idx_audit_escalated
    ON invariant_audit_log (escalated, time DESC)
    WHERE escalated = TRUE;

-- ─── federation_commits ───────────────────────────────────────────────────────
-- One row per Elixir quorum commit of a SnapshotProposal.
-- proposal_id is globally unique.
-- gate_status reflects the OCaml gate result at proposal admission time
-- ("valid" or "uncertain" — "invalid" proposals never reach commit).

CREATE TABLE IF NOT EXISTS federation_commits (
    time                TIMESTAMPTZ NOT NULL,
    proposal_id         TEXT        NOT NULL,
    simulation_id       TEXT        NOT NULL,
    step_index          BIGINT      NOT NULL,
    state_hash          TEXT        NOT NULL,
    aggregate_signature BYTEA,       -- BLS aggregate sig (placeholder in Phase 2C)
    quorum_size         INTEGER     NOT NULL DEFAULT 1,
    votes_received      INTEGER     NOT NULL DEFAULT 1,
    gate_status         TEXT        NOT NULL DEFAULT 'valid',

    CONSTRAINT commits_proposal_nonempty CHECK (length(proposal_id) > 0),
    CONSTRAINT commits_hash_nonempty     CHECK (length(state_hash) = 16),
    CONSTRAINT commits_gate_valid        CHECK (gate_status IN ('valid', 'uncertain')),
    CONSTRAINT commits_quorum_positive   CHECK (quorum_size >= 1),
    CONSTRAINT commits_votes_sane        CHECK (votes_received >= 1 AND votes_received <= quorum_size)
);

SELECT create_hypertable(
    'federation_commits',
    'time',
    if_not_exists => TRUE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_commits_proposal
    ON federation_commits (proposal_id);

CREATE INDEX IF NOT EXISTS idx_commits_sim_step
    ON federation_commits (simulation_id, step_index DESC);

-- ─── schema_migrations ────────────────────────────────────────────────────────
-- Version tracking for migration management.

CREATE TABLE IF NOT EXISTS schema_migrations (
    version    TEXT        NOT NULL PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT
);

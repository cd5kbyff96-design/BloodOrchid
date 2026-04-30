-- storage/timescaledb/migrations/001_init.up.sql
-- Vail Iris — Initial Schema Migration
-- Phase 2C: MVES-aligned hypertables.
-- Reversible via: 001_init.down.sql
--
-- Rules:
--   - Entire migration is wrapped in a single transaction
--   - All DDL is idempotent (IF NOT EXISTS throughout)
--   - No DML in migration files
--   - No product-specific logic; MVES-aligned tables only in Phase 2C
--   - TimescaleDB extension must be available before this migration runs
--     (pre-installed in the Docker image; see infra/k8s/timescaledb/)
--
-- CI gate: psql --no-psqlrc -f 001_init.up.sql against a clean TimescaleDB
-- container must exit 0 on first run and on re-run (idempotency test).

BEGIN;

-- ─── Guard: ensure TimescaleDB is present ────────────────────────────────────

CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- ─── simulation_trajectories ─────────────────────────────────────────────────

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

CREATE TABLE IF NOT EXISTS invariant_audit_log (
    time               TIMESTAMPTZ NOT NULL,
    pipeline           TEXT        NOT NULL,
    simulation_id      TEXT        NOT NULL,
    step_index         BIGINT      NOT NULL,
    gate_result        TEXT        NOT NULL,
    confidence         REAL,
    violation_trace    TEXT,
    fallback_policy_id TEXT,
    escalated          BOOLEAN     NOT NULL DEFAULT FALSE,

    CONSTRAINT audit_gate_valid     CHECK (gate_result IN ('valid', 'uncertain', 'invalid')),
    CONSTRAINT audit_step_nonneg    CHECK (step_index >= 0),
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

CREATE TABLE IF NOT EXISTS federation_commits (
    time                TIMESTAMPTZ NOT NULL,
    proposal_id         TEXT        NOT NULL,
    simulation_id       TEXT        NOT NULL,
    step_index          BIGINT      NOT NULL,
    state_hash          TEXT        NOT NULL,
    aggregate_signature BYTEA,
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

CREATE TABLE IF NOT EXISTS schema_migrations (
    version     TEXT        NOT NULL PRIMARY KEY,
    applied_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT
);

INSERT INTO schema_migrations (version, description)
VALUES (
    '001',
    'Initialize MVES-aligned TimescaleDB hypertables — simulation_trajectories, invariant_audit_log, federation_commits (Phase 2C)'
)
ON CONFLICT (version) DO NOTHING;

COMMIT;

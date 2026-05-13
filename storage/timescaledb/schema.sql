-- storage/timescaledb/schema.sql
-- Vail Iris Blood Orchid - TimescaleDB Schema
-- Comprehensive database schema for MVES simulation pipeline

-- Requires: TimescaleDB extension >= 2.13, PostgreSQL >= 15

-- ─── Extensions ──────────────────────────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ─── Configuration Tables ───────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS simulation_config (
    config_id       UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id   TEXT           NOT NULL UNIQUE,
    solver_kind     TEXT           NOT NULL,
    width           INTEGER        NOT NULL,
    height          INTEGER        NOT NULL,
    dx              DOUBLE PRECISION NOT NULL,
    dy              DOUBLE PRECISION NOT NULL,
    dt              DOUBLE PRECISION NOT NULL,
    alpha           DOUBLE PRECISION NOT NULL,
    max_steps       BIGINT         NOT NULL,
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    
    CONSTRAINT config_positive_dims CHECK (width > 0 AND height > 0),
    CONSTRAINT config_positive_dt CHECK (dt > 0)
);

CREATE TABLE IF NOT EXISTS solver_registry (
    solver_id       TEXT           PRIMARY KEY,
    solver_name     TEXT           NOT NULL,
    solver_kind     TEXT           NOT NULL,
    version         TEXT           NOT NULL,
    description     TEXT,
    config_schema   JSONB,
    enabled         BOOLEAN        NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT NOW()
);

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
    field_stats      JSONB,
    metadata         JSONB,
    
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

CREATE INDEX IF NOT EXISTS idx_traj_solver_time
    ON simulation_trajectories (solver_kind, time DESC);

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
    processing_time_ms INTEGER,
    
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

CREATE INDEX IF NOT EXISTS idx_audit_pipeline_time
    ON invariant_audit_log (pipeline, time DESC);

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
    voter_signatures    JSONB,
    commit_latency_ms   INTEGER,
    
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

-- ─── cve_transforms ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS cve_transforms (
    time              TIMESTAMPTZ NOT NULL,
    transform_id      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id     TEXT        NOT NULL,
    step_index        BIGINT      NOT NULL,
    scene_data        BYTEA       NOT NULL,
    vertex_count      INTEGER     NOT NULL,
    triangle_count    INTEGER     NOT NULL,
    transform_time_ms INTEGER,
    metadata          JSONB
);

SELECT create_hypertable(
    'cve_transforms',
    'time',
    if_not_exists => TRUE
);

CREATE INDEX IF NOT EXISTS idx_cve_sim_step
    ON cve_transforms (simulation_id, step_index DESC);

-- ─── runtime_metrics ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS runtime_metrics (
    time              TIMESTAMPTZ NOT NULL,
    simulation_id     TEXT        NOT NULL,
    metric_name       TEXT        NOT NULL,
    metric_value      DOUBLE PRECISION,
    metric_metadata   JSONB
);

SELECT create_hypertable(
    'runtime_metrics',
    'time',
    if_not_exists => TRUE
);

CREATE INDEX IF NOT EXISTS idx_metrics_sim_name
    ON runtime_metrics (simulation_id, metric_name, time DESC);

-- ─── system_logs ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS system_logs (
    time              TIMESTAMPTZ NOT NULL,
    log_level         TEXT        NOT NULL,
    component         TEXT        NOT NULL,
    message           TEXT        NOT NULL,
    metadata          JSONB,
    trace_id          TEXT
);

SELECT create_hypertable(
    'system_logs',
    'time',
    if_not_exists => TRUE
);

CREATE INDEX IF NOT EXISTS idx_logs_level_time
    ON system_logs (log_level, time DESC);

CREATE INDEX IF NOT EXISTS idx_logs_trace
    ON system_logs (trace_id)
    WHERE trace_id IS NOT NULL;

-- ─── schema_migrations ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS schema_migrations (
    version    TEXT        NOT NULL PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT,
    checksum   TEXT
);

-- ─── Functions ───────────────────────────────────────────────────────────────

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION compute_simulation_stats(state_vector BYTEA)
RETURNS JSONB AS $$
DECLARE
    stats JSONB;
BEGIN
    stats = jsonb_build_object(
        'min', 0.0,
        'max', 1.0,
        'mean', 0.5,
        'std', 0.1,
        'nan_count', 0,
        'inf_count', 0
    );
    RETURN stats;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION stable_hash64(data BYTEA)
RETURNS TEXT AS $$
BEGIN
    RETURN encode(digest(data, 'fnv'), 'hex');
END;
$$ LANGUAGE plpgsql;

-- ─── Triggers ────────────────────────────────────────────────────────────────

CREATE TRIGGER update_simulation_config_updated_at
    BEFORE UPDATE ON simulation_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_trajectory_field_stats
    BEFORE INSERT ON simulation_trajectories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ─── Views ─────────────────────────────────────────────────────────────────────

CREATE OR REPLACE VIEW latest_simulation_states AS
SELECT DISTINCT ON (simulation_id)
    simulation_id,
    step_index,
    simulation_time,
    state_hash,
    solver_kind,
    time as last_update
FROM simulation_trajectories
ORDER BY simulation_id, time DESC;

CREATE OR REPLACE VIEW simulation_summary AS
SELECT
    simulation_id,
    count(*) as total_steps,
    min(time) as first_step,
    max(time) as last_step,
    max(simulation_time) as final_time,
    avg(simulation_time - lag(simulation_time) OVER (PARTITION BY simulation_id ORDER BY time)) as avg_dt
FROM simulation_trajectories
GROUP BY simulation_id;
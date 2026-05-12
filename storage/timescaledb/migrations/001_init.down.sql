-- storage/timescaledb/migrations/001_init.down.sql
-- Vail Iris — Rollback for initial schema migration

BEGIN;

DROP INDEX IF EXISTS idx_traj_sim_step;
DROP INDEX IF EXISTS idx_traj_product_time;
DROP TABLE IF EXISTS simulation_trajectories;

DROP INDEX IF EXISTS idx_audit_sim_gate;
DROP INDEX IF EXISTS idx_audit_escalated;
DROP TABLE IF EXISTS invariant_audit_log;

DROP INDEX IF EXISTS idx_commits_proposal;
DROP INDEX IF EXISTS idx_commits_sim_step;
DROP TABLE IF EXISTS federation_commits;

DELETE FROM schema_migrations WHERE version = '001';

COMMIT;
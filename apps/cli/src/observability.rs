use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use boundary_runtime::proto::{GeometryScene, SimulationState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub trace_id: String,
    pub simulation_id: String,
    pub solver_kind: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub initial_state_hash: u64,
    pub final_state_hash: Option<u64>,
    pub lineage_merkle_root: Option<u64>,
    pub steps: Vec<StepRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step_index: u64,
    pub input_state_hash: u64,
    pub output_state_hash: u64,
    pub transform_input_hash: Option<u64>,
    pub transform_output_hash: Option<u64>,
    pub scene_hash: Option<u64>,
    pub kernel_duration_ms: u64,
    pub transform_duration_ms: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateLineage {
    pub simulation_id: String,
    pub initial_hash: u64,
    pub links: Vec<ChainLink>,
    pub final_hash: u64,
    pub merkle_root: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainLink {
    pub step: u64,
    pub prev_hash: u64,
    pub current_hash: u64,
}

pub struct TraceLogger {
    trace: ExecutionTrace,
    current_step_start: u64,
    config: TraceConfig,
}

#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub enabled: bool,
    pub save_steps: bool,
    pub save_scenes: bool,
    pub output_path: Option<PathBuf>,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            save_steps: true,
            save_scenes: false,
            output_path: None,
        }
    }
}

impl TraceLogger {
    pub fn new(simulation_id: &str, solver_kind: &str) -> Self {
        Self {
            trace: ExecutionTrace {
                trace_id: generate_trace_id(),
                simulation_id: simulation_id.to_string(),
                solver_kind: solver_kind.to_string(),
                started_at: current_timestamp(),
                completed_at: None,
                initial_state_hash: 0,
                final_state_hash: None,
                lineage_merkle_root: None,
                steps: Vec::new(),
            },
            current_step_start: current_timestamp(),
            config: TraceConfig::default(),
        }
    }

    pub fn with_config(config: TraceConfig, simulation_id: &str, solver_kind: &str) -> Self {
        Self {
            trace: ExecutionTrace {
                trace_id: generate_trace_id(),
                simulation_id: simulation_id.to_string(),
                solver_kind: solver_kind.to_string(),
                started_at: current_timestamp(),
                completed_at: None,
                initial_state_hash: 0,
                final_state_hash: None,
                lineage_merkle_root: None,
                steps: Vec::new(),
            },
            current_step_start: current_timestamp(),
            config,
        }
    }

    pub fn set_initial_hash(&mut self, hash: u64) {
        self.trace.initial_state_hash = hash;
    }

    pub fn record_step(
        &mut self,
        step_index: u64,
        input_state_hash: u64,
        output_state_hash: u64,
        kernel_duration_ms: u64,
    ) {
        self.trace.steps.push(StepRecord {
            step_index,
            input_state_hash,
            output_state_hash,
            transform_input_hash: None,
            transform_output_hash: None,
            scene_hash: None,
            kernel_duration_ms,
            transform_duration_ms: 0,
            timestamp: current_timestamp(),
        });
    }

    pub fn record_transform(
        &mut self,
        step_index: u64,
        input_hash: u64,
        output_hash: u64,
        scene_hash: u64,
        duration_ms: u64,
    ) {
        if let Some(step) = self.trace.steps.iter_mut().find(|s| s.step_index == step_index) {
            step.transform_input_hash = Some(input_hash);
            step.transform_output_hash = Some(output_hash);
            step.scene_hash = Some(scene_hash);
            step.transform_duration_ms = duration_ms;
        }
    }

    pub fn finalize(&mut self, final_state_hash: u64) {
        self.trace.completed_at = Some(current_timestamp());
        self.trace.final_state_hash = Some(final_state_hash);
        self.trace.lineage_merkle_root = Some(self.compute_merkle_root());
    }

    pub fn compute_merkle_root(&self) -> u64 {
        if self.trace.steps.is_empty() {
            return self.trace.initial_state_hash;
        }

        let mut hashes: Vec<u64> = self.trace.steps.iter()
            .map(|s| s.output_state_hash)
            .collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                hashes.push(hashes.last().copied().unwrap_or(0));
            }

            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = chunk[0].wrapping_mul(31).wrapping_add(chunk[1]);
                next_level.push(combined);
            }
            hashes = next_level;
        }

        hashes.first().copied().unwrap_or(0)
    }

    pub fn get_trace(&self) -> &ExecutionTrace {
        &self.trace
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.trace)
            .map_err(|e| e.to_string())?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn compute_lineage(&self) -> StateLineage {
        let mut links = Vec::new();
        let mut prev_hash = self.trace.initial_state_hash;

        for step in &self.trace.steps {
            links.push(ChainLink {
                step: step.step_index,
                prev_hash,
                current_hash: step.output_state_hash,
            });
            prev_hash = step.output_state_hash;
        }

        let final_hash = self.trace.final_state_hash.unwrap_or(prev_hash);

        StateLineage {
            simulation_id: self.trace.simulation_id.clone(),
            initial_hash: self.trace.initial_state_hash,
            links,
            final_hash,
            merkle_root: self.compute_merkle_root(),
        }
    }

    pub fn total_kernel_time(&self) -> u64 {
        self.trace.steps.iter().map(|s| s.kernel_duration_ms).sum()
    }

    pub fn total_transform_time(&self) -> u64 {
        self.trace.steps.iter().map(|s| s.transform_duration_ms).sum()
    }

    pub fn step_count(&self) -> usize {
        self.trace.steps.len()
    }
}

pub struct ReplayManager {
    traces: HashMap<String, ExecutionTrace>,
}

impl ReplayManager {
    pub fn new() -> Self {
        Self {
            traces: HashMap::new(),
        }
    }

    pub fn load_trace(&mut self, path: &PathBuf) -> Result<String, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let trace: ExecutionTrace = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let id = trace.trace_id.clone();
        self.traces.insert(id.clone(), trace);
        Ok(id)
    }

    pub fn get_trace(&self, trace_id: &str) -> Option<&ExecutionTrace> {
        self.traces.get(trace_id)
    }

    pub fn verify_lineage(&self, trace_id: &str) -> Result<bool, String> {
        let trace = self.traces.get(trace_id).ok_or("trace not found")?;

        if trace.steps.is_empty() {
            return Ok(true);
        }

        let mut expected_prev = trace.initial_state_hash;

        for step in &trace.steps {
            if step.input_state_hash != expected_prev {
                return Ok(false);
            }
            expected_prev = step.output_state_hash;
        }

        if let Some(final_hash) = trace.final_state_hash {
            return Ok(expected_prev == final_hash);
        }

        Ok(true)
    }

    pub fn get_step_info(&self, trace_id: &str, step: u64) -> Option<StepRecord> {
        self.traces.get(trace_id)
            .and_then(|t| t.steps.iter().find(|s| s.step_index == step))
            .cloned()
    }

    pub fn list_traces(&self) -> Vec<(String, String)> {
        self.traces.iter()
            .map(|(id, t)| (id.clone(), t.simulation_id.clone()))
            .collect()
    }

    pub fn get_summary(&self, trace_id: &str) -> Option<TraceSummary> {
        let trace = self.traces.get(trace_id)?;

        Some(TraceSummary {
            simulation_id: trace.simulation_id.clone(),
            step_count: trace.steps.len(),
            total_kernel_ms: trace.steps.iter().map(|s| s.kernel_duration_ms).sum(),
            total_transform_ms: trace.steps.iter().map(|s| s.transform_duration_ms).sum(),
            initial_hash: trace.initial_state_hash,
            final_hash: trace.final_state_hash,
        })
    }
}

impl Default for ReplayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TraceSummary {
    pub simulation_id: String,
    pub step_count: usize,
    pub total_kernel_ms: u64,
    pub total_transform_ms: u64,
    pub initial_hash: u64,
    pub final_hash: Option<u64>,
}

fn generate_trace_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("trace-{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn compute_state_hash(state: &SimulationState) -> u64 {
    let bytes = state.encode();
    fnv_hash(&bytes)
}

pub fn compute_scene_hash(scene: &GeometryScene) -> u64 {
    let bytes = scene.encode();
    fnv_hash(&bytes)
}

fn fnv_hash(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundary_runtime::proto::FieldTensor;

    fn make_test_state() -> SimulationState {
        SimulationState {
            simulation_id: "test-sim".to_string(),
            solver_kind: "heat_reference".to_string(),
            step_index: 0,
            simulation_time: 0.0,
            primary_field: Some(FieldTensor {
                field_name: "test".to_string(),
                field_kind: "scalar".to_string(),
                width: 3,
                height: 3,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![0.0_f32; 9],
            }),
        }
    }

    #[test]
    fn test_trace_creation() {
        let logger = TraceLogger::new("test-sim", "heat_reference");
        assert_eq!(logger.trace.steps.len(), 0);
    }

    #[test]
    fn test_trace_step_recording() {
        let mut logger = TraceLogger::new("test-sim", "heat_reference");
        logger.set_initial_hash(0x123);

        logger.record_step(1, 0x123, 0x456, 10);
        logger.record_step(2, 0x456, 0x789, 12);

        assert_eq!(logger.trace.steps.len(), 2);
        assert_eq!(logger.trace.steps[0].step_index, 1);
    }

    #[test]
    fn test_lineage_computation() {
        let mut logger = TraceLogger::new("test-sim", "heat_reference");
        logger.set_initial_hash(0x100);
        logger.record_step(1, 0x100, 0x200, 10);
        logger.record_step(2, 0x200, 0x300, 10);
        logger.finalize(0x300);

        let lineage = logger.compute_lineage();

        assert_eq!(lineage.initial_hash, 0x100);
        assert_eq!(lineage.final_hash, 0x300);
        assert_eq!(lineage.links.len(), 2);
    }

    #[test]
    fn test_replay_manager_load() {
        let mut manager = ReplayManager::new();
        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("test_trace.json");

        let mut logger = TraceLogger::new("test-sim", "heat");
        logger.set_initial_hash(0x100);
        logger.record_step(1, 0x100, 0x200, 10);
        logger.finalize(0x200);

        logger.save(&trace_path).unwrap();

        let trace_id = manager.load_trace(&trace_path).unwrap();
        let trace = manager.get_trace(&trace_id).unwrap();

        assert_eq!(trace.simulation_id, "test-sim");

        std::fs::remove_file(trace_path).ok();
    }

    #[test]
    fn test_lineage_verification() {
        let mut manager = ReplayManager::new();
        let temp_dir = std::env::temp_dir();
        let trace_path = temp_dir.join("verify_trace.json");

        let mut logger = TraceLogger::new("test-sim", "heat");
        logger.set_initial_hash(0x100);
        logger.record_step(1, 0x100, 0x200, 10);
        logger.record_step(2, 0x200, 0x300, 10);
        logger.finalize(0x300);

        logger.save(&trace_path).unwrap();

        let trace_id = manager.load_trace(&trace_path).unwrap();
        let valid = manager.verify_lineage(&trace_id).unwrap();

        assert!(valid);

        std::fs::remove_file(trace_path).ok();
    }

    #[test]
    fn test_merkle_root_single_step() {
        let mut logger = TraceLogger::new("test", "heat");
        logger.set_initial_hash(0x100);
        logger.record_step(1, 0x100, 0x200, 10);
        logger.finalize(0x200);

        let root = logger.compute_merkle_root();
        assert_eq!(root, 0x200);
    }
}
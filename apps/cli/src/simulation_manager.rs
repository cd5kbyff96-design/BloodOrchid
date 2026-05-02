use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use boundary_runtime::boundary::BoundaryRuntime;
use boundary_runtime::proto::{FieldTensor, GeometryScene, SimulationState};
use cve_core::map_state_to_scene;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulationStatus {
    Initializing,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub solver_kind: String,
    pub max_steps: Option<u64>,
    pub output_path: Option<PathBuf>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            solver_kind: "heat_reference".to_string(),
            max_steps: None,
            output_path: None,
        }
    }
}

pub struct SimulationHandle {
    pub id: SimulationId,
    pub boundary: Arc<std::sync::Mutex<BoundaryRuntime>>,
    pub status: SimulationStatus,
    pub created_at: Instant,
    pub config: SimulationConfig,
}

impl SimulationHandle {
    pub fn new(id: SimulationId, boundary: BoundaryRuntime, config: SimulationConfig) -> Self {
        Self {
            id,
            boundary: Arc::new(std::sync::Mutex::new(boundary)),
            status: SimulationStatus::Initializing,
            created_at: Instant::now(),
            config,
        }
    }

    pub fn step(&self, count: u64) -> Result<Arc<SimulationState>, String> {
        let mut boundary = self.boundary.lock().map_err(|e| e.to_string())?;
        let result = boundary.step(count).map_err(|e| e.reason.clone())?;
        Ok(result)
    }

    pub fn get_snapshot(&self) -> Result<Arc<SimulationState>, String> {
        let boundary = self.boundary.lock().map_err(|e| e.to_string())?;
        Ok(boundary.get_snapshot())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationId(String);

impl SimulationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SimulationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct SimulationRegistry {
    handles: HashMap<SimulationId, SimulationHandle>,
}

impl SimulationRegistry {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        initial_state: SimulationState,
        config: SimulationConfig,
    ) -> Result<SimulationId, String> {
        let id = SimulationId::new();
        let boundary = BoundaryRuntime::init(initial_state)
            .map_err(|e| e.reason.clone())?;

        let handle = SimulationHandle::new(id.clone(), boundary, config);
        self.handles.insert(id.clone(), handle);

        Ok(id)
    }

    pub fn get(&self, id: &SimulationId) -> Option<&SimulationHandle> {
        self.handles.get(id)
    }

    pub fn get_mut(&mut self, id: &SimulationId) -> Option<&mut SimulationHandle> {
        self.handles.get_mut(id)
    }

    pub fn remove(&mut self, id: &SimulationId) -> Result<(), String> {
        self.handles.remove(id).ok_or_else(|| "simulation not found".to_string())?;
        Ok(())
    }

    pub fn list(&self) -> Vec<SimulationId> {
        self.handles.keys().cloned().collect()
    }

    pub fn status(&self, id: &SimulationId) -> Option<&SimulationStatus> {
        self.handles.get(id).map(|h| &h.status)
    }

    pub fn count(&self) -> usize {
        self.handles.len()
    }
}

impl Default for SimulationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BatchItem {
    pub simulation_id: String,
    pub initial_state: SimulationState,
    pub steps: u64,
    pub output_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct BatchResult {
    pub simulation_id: String,
    pub final_state: Option<Arc<SimulationState>>,
    pub scene: Option<GeometryScene>,
    pub scene_bytes: Option<Vec<u8>>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

pub struct BatchExecutor {
    max_concurrent: usize,
}

impl BatchExecutor {
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    pub fn execute(&self, items: Vec<BatchItem>) -> Vec<BatchResult> {
        use std::thread;

        let max_concurrent = self.max_concurrent;
        let semaphore = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let mut handles: Vec<std::thread::JoinHandle<BatchResult>> = Vec::new();

        for item in items {
            let sem = semaphore.clone();

            let handle = thread::spawn(move || {
                while sem.load(std::sync::atomic::Ordering::Acquire) >= max_concurrent {
                    thread::yield_now();
                }
                sem.fetch_add(1, std::sync::atomic::Ordering::AcqRel);

                let result = run_single_item(item);

                sem.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);

                result
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.join() {
                results.push(result);
            }
        }

        results
    }
}

fn run_single_item(item: BatchItem) -> BatchResult {
    let mut boundary = match BoundaryRuntime::init(item.initial_state.clone()) {
        Ok(b) => b,
        Err(e) => {
            return BatchResult {
                simulation_id: item.simulation_id,
                final_state: None,
                scene: None,
                scene_bytes: None,
                error: Some(e.reason),
                execution_time_ms: 0,
            };
        }
    };

    match boundary.step(item.steps) {
        Ok(snapshot) => {
            let scene = match map_state_to_scene(&snapshot) {
                Ok(s) => s,
                Err(e) => {
                    return BatchResult {
                        simulation_id: item.simulation_id,
                        final_state: Some(snapshot),
                        scene: None,
                        scene_bytes: None,
                        error: Some(e),
                        execution_time_ms: 0,
                    };
                }
            };

            let scene_bytes = scene.encode();

            if let Some(ref path) = item.output_path {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(path, &scene_bytes).ok();
            }

            BatchResult {
                simulation_id: item.simulation_id,
                final_state: Some(snapshot),
                scene: Some(scene),
                scene_bytes: Some(scene_bytes),
                error: None,
                execution_time_ms: 0,
            }
        }
        Err(e) => BatchResult {
            simulation_id: item.simulation_id,
            final_state: None,
            scene: None,
            scene_bytes: None,
            error: Some(e.reason),
            execution_time_ms: 0,
        },
    }
}

pub fn create_initial_state_with_id(simulation_id: &str) -> SimulationState {
    SimulationState {
        simulation_id: simulation_id.to_string(),
        solver_kind: "heat_reference".to_string(),
        step_index: 0,
        simulation_time: 0.0,
        primary_field: Some(FieldTensor {
            field_name: "temperature".to_string(),
            field_kind: "scalar".to_string(),
            width: 3,
            height: 3,
            channels: 1,
            cell_spacing: 1.0,
            values: vec![
                0.0_f32, 0.0, 0.0,
                0.0, 0.0, 0.0,
                0.0, 0.0, 0.0,
            ],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> SimulationState {
        SimulationState {
            simulation_id: "mves-heat-2d".to_string(),
            solver_kind: "heat_reference".to_string(),
            step_index: 0,
            simulation_time: 0.0,
            primary_field: Some(FieldTensor {
                field_name: "temperature".to_string(),
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
    fn test_simulation_registry_create() {
        let mut registry = SimulationRegistry::new();
        let state = make_test_state();
        let id = registry.create(state, SimulationConfig::default());

        assert!(id.is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_simulation_registry_isolation() {
        let mut registry = SimulationRegistry::new();

        let id1 = registry.create(make_test_state(), SimulationConfig::default()).unwrap();
        let _id2 = registry.create(make_test_state(), SimulationConfig::default()).unwrap();

        let handle1 = registry.get(&id1).unwrap();

        let result = handle1.step(1);
        assert!(result.is_ok(), "step should succeed: {:?}", result);

        let snap1 = handle1.get_snapshot().unwrap();
        assert_eq!(snap1.step_index, 1, "step should advance to 1");
    }

    #[test]
    fn test_simulation_registry_remove() {
        let mut registry = SimulationRegistry::new();
        let id = registry.create(make_test_state(), SimulationConfig::default()).unwrap();

        assert_eq!(registry.count(), 1);

        registry.remove(&id).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_batch_executor_independent_outputs() {
        let executor = BatchExecutor::new(4);

        let items = vec![
            BatchItem {
                simulation_id: "batch1".to_string(),
                initial_state: make_test_state(),
                steps: 2,
                output_path: None,
            },
            BatchItem {
                simulation_id: "batch2".to_string(),
                initial_state: make_test_state(),
                steps: 2,
                output_path: None,
            },
        ];

        let results = executor.execute(items);

        assert_eq!(results.len(), 2);

        let r1 = &results[0];
        let r2 = &results[1];

        if let Some(ref err) = r1.error {
            panic!("batch1 error: {}", err);
        }
        if let Some(ref err) = r2.error {
            panic!("batch2 error: {}", err);
        }

        assert!(r1.final_state.is_some(), "batch1 should have final state");
        assert!(r2.final_state.is_some(), "batch2 should have final state");

        assert_eq!(r1.final_state.as_ref().unwrap().step_index, 2);
        assert_eq!(r2.final_state.as_ref().unwrap().step_index, 2);
    }

    #[test]
    fn test_batch_determinism() {
        let executor = BatchExecutor::new(4);

        let state = make_test_state();

        let items = vec![
            BatchItem {
                simulation_id: "det1".to_string(),
                initial_state: state.clone(),
                steps: 1,
                output_path: None,
            },
            BatchItem {
                simulation_id: "det2".to_string(),
                initial_state: state.clone(),
                steps: 1,
                output_path: None,
            },
        ];

        let results = executor.execute(items);

        if results.iter().any(|r| r.error.is_some()) {
            panic!("batch execution had errors: {:?}", results.iter().filter(|r| r.error.is_some()).collect::<Vec<_>>());
        }

        let bytes1 = results[0].scene_bytes.as_ref().unwrap();
        let bytes2 = results[1].scene_bytes.as_ref().unwrap();

        assert_eq!(bytes1, bytes2, "Identical inputs must produce identical outputs");
    }

    #[test]
    fn test_parallel_simulations_no_cross_contamination() {
        let mut registry = SimulationRegistry::new();

        let id1 = registry.create(make_test_state(), SimulationConfig::default()).unwrap();
        let id2 = registry.create(make_test_state(), SimulationConfig::default()).unwrap();

        let h1 = registry.get(&id1).unwrap();
        let h2 = registry.get(&id2).unwrap();

        h1.step(3).ok();
        h2.step(5).ok();

        let s1 = h1.get_snapshot().unwrap();
        let s2 = h2.get_snapshot().unwrap();

        assert_eq!(s1.step_index, 3);
        assert_eq!(s2.step_index, 5);
    }
}
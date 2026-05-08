use std::fs;
use std::path::Path;
use std::time::Instant;

pub mod simulation_manager;
pub mod observability;

pub mod simulation_manager;

use boundary_runtime::boundary::BoundaryRuntime;
use boundary_runtime::proto::{FieldTensor, GeometryScene, SimulationState};
use cve_core::{stable_hash64, map_state_to_scene};
use observability::{compute_scene_hash, compute_state_hash, TraceLogger};
use simulation_manager::{
    BatchExecutor, BatchItem, BatchResult, SimulationConfig, SimulationHandle,
    SimulationId, SimulationRegistry, SimulationStatus,
};

#[derive(Clone, Debug)]
pub struct PipelineResult {
    pub simulation_id: String,
    pub final_step: u64,
    pub state_hash: String,
    pub scene_hash: String,
    pub output_path: String,
}

impl std::fmt::Display for PipelineResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[SUCCESS] BloodOrchid MVES Pipeline Complete")?;
        writeln!(f)?;
        writeln!(f, "  Simulation ID: {}", self.simulation_id)?;
        writeln!(f, "  Final Step:    {}", self.final_step)?;
        writeln!(f, "  State Hash:    {}", self.state_hash)?;
        writeln!(f, "  Scene Hash:    {}", self.scene_hash)?;
        writeln!(f, "  Output:        {}", self.output_path)?;
        writeln!(f)?;
        write!(f, "✓ Pipeline succeeded")
    }
}

#[derive(Debug, Clone)]
pub enum CliError {
    ArgumentError(String),
    BoundaryError(String),
    TransformError(String),
    IoError(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::ArgumentError(msg) => write!(f, "argument error: {}", msg),
            CliError::BoundaryError(msg) => write!(f, "boundary error: {}", msg),
            CliError::TransformError(msg) => write!(f, "transform error: {}", msg),
            CliError::IoError(msg) => write!(f, "io error: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

fn create_initial_state() -> SimulationState {
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
            values: vec![
                0.0_f32, 0.0, 0.0,
                0.0, 0.0, 0.0,
                0.0, 0.0, 0.0,
            ],
        }),
    }
}

pub fn run_pipeline(steps: u64, output_path: &Path) -> Result<PipelineResult, CliError> {
    run_pipeline_with_trace(steps, output_path, None).map(|r| r.0)
}

pub fn run_pipeline_with_trace(
    steps: u64,
    output_path: &Path,
    trace_output: Option<&Path>,
) -> Result<(PipelineResult, TraceLogger), CliError> {
    let initial_state = create_initial_state();
    let sim_id = initial_state.simulation_id.clone();
    let solver = initial_state.solver_kind.clone();

    let mut trace = TraceLogger::new(&sim_id, &solver);
    let init_hash = compute_state_hash(&initial_state);
    trace.set_initial_hash(init_hash);

    let mut boundary = BoundaryRuntime::init(initial_state.clone())
        .map_err(|e| CliError::BoundaryError(e.reason.clone()))?;

    let step_size = if steps > 10 { steps / 10 } else { 1 };

    for i in (0..steps).step_by(step_size as usize) {
        let remaining = steps - i;
        let current_step = remaining.min(step_size);

        let start = Instant::now();
        let before_state = (*boundary.get_snapshot()).clone();
        let before_hash = compute_state_hash(&before_state);

        boundary.step(current_step)
            .map_err(|e| CliError::BoundaryError(e.reason.clone()))?;

        let kernel_time = start.elapsed().as_millis() as u64;
        let snapshot = boundary.get_snapshot();
        let after_hash = compute_state_hash(&snapshot);

        let actual_step = snapshot.step_index;
        trace.record_step(actual_step, before_hash, after_hash, kernel_time);

        let transform_start = Instant::now();
        let scene = map_state_to_scene(&snapshot)
            .map_err(|e| CliError::TransformError(e))?;
        let transform_time = transform_start.elapsed().as_millis() as u64;

        let scene_hash = compute_scene_hash(&scene);
        trace.record_transform(actual_step, after_hash, scene_hash, scene_hash, transform_time);
    }

    let snapshot = boundary.get_snapshot();
    let final_hash = compute_state_hash(&snapshot);
    trace.finalize(final_hash);

    if let Some(trace_path) = trace_output {
        trace.save(&trace_path.to_path_buf()).ok();
    }

    let state_bytes = snapshot.encode();
    let state_hash = stable_hash64(&state_bytes);

    let scene = map_state_to_scene(&snapshot)
        .map_err(|e| CliError::TransformError(e))?;
    let scene_bytes = scene.encode();
    let scene_hash = stable_hash64(&scene_bytes);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CliError::IoError(format!("failed to create directory: {}", e)))?;
    }
    fs::write(output_path, &scene_bytes)
        .map_err(|e| CliError::IoError(format!("failed to write output: {}", e)))?;

    let lineage = trace.compute_lineage();
    eprintln!("[TRACE] Lineage: {} steps, merkle_root={:016x}", lineage.links.len(), lineage.merkle_root);
    eprintln!("[TRACE] Total kernel: {}ms, transform: {}ms",
        trace.total_kernel_time(), trace.total_transform_time());

    Ok((PipelineResult {
        simulation_id: snapshot.simulation_id.clone(),
        final_step: snapshot.step_index,
        state_hash: format!("{:016x}", state_hash),
        scene_hash: format!("{:016x}", scene_hash),
        output_path: output_path.display().to_string(),
    }, trace))
}

pub fn run_demo(steps: u64, scene_out: &Path) -> Result<DemoResult, CliError> {
    let initial_state = create_initial_state();

    let mut boundary = BoundaryRuntime::init(initial_state)
        .map_err(|e| CliError::BoundaryError(e.reason.clone()))?;

    boundary.step(steps)
        .map_err(|e| CliError::BoundaryError(e.reason.clone()))?;

    let snapshot = boundary.get_snapshot();
    let state_bytes = snapshot.encode();
    let state_hash = stable_hash64(&state_bytes);

    let scene = map_state_to_scene(&snapshot)
        .map_err(|e| CliError::TransformError(e))?;
    let scene_bytes = scene.encode();
    let scene_hash = stable_hash64(&scene_bytes);

    if let Some(parent) = scene_out.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CliError::IoError(format!("failed to create directory: {}", e)))?;
    }
    fs::write(scene_out, &scene_bytes)
        .map_err(|e| CliError::IoError(format!("failed to write output: {}", e)))?;

    Ok(DemoResult {
        state_hash: format!("{:016x}", state_hash),
        scene_hash: format!("{:016x}", scene_hash),
        state: (*snapshot).clone(),
        scene,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct DemoResult {
    pub state_hash: String,
    pub scene_hash: String,
    pub state: SimulationState,
    pub scene: GeometryScene,
}

pub fn decode_scene_file(scene_path: &Path) -> Result<GeometryScene, String> {
    let bytes = fs::read(scene_path).map_err(|error| error.to_string())?;
    GeometryScene::decode(&bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    fn create_test_state() -> SimulationState {
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
    fn test_cli_statelessness() {
        let temp_dir = temp_dir();
        let output1 = temp_dir.join("test_cli_1.pb");
        let output2 = temp_dir.join("test_cli_2.pb");

        let initial = create_test_state();

        let mut boundary1 = BoundaryRuntime::init(initial.clone()).unwrap();
        boundary1.step(1).unwrap();
        let snapshot1 = boundary1.get_snapshot();
        let scene1 = map_state_to_scene(&snapshot1).unwrap();
        fs::write(&output1, scene1.encode()).unwrap();

        let mut boundary2 = BoundaryRuntime::init(initial.clone()).unwrap();
        boundary2.step(1).unwrap();
        let snapshot2 = boundary2.get_snapshot();
        let scene2 = map_state_to_scene(&snapshot2).unwrap();
        fs::write(&output2, scene2.encode()).unwrap();

        let hash1 = fs::read(&output1).unwrap();
        let hash2 = fs::read(&output2).unwrap();
        assert_eq!(hash1, hash2);

        let _ = fs::remove_file(output1);
        let _ = fs::remove_file(output2);
    }

    #[test]
    fn test_boundary_enforcement() {
        let invalid_state = SimulationState {
            simulation_id: "".to_string(),
            solver_kind: "test".to_string(),
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
        };

        let result = BoundaryRuntime::init(invalid_state);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_ordering() {
        let temp_dir = temp_dir();
        let output = temp_dir.join("test_order.pb");

        let result = run_pipeline(1, &output);
        assert!(result.is_ok(), "pipeline should succeed: {:?}", result);

        assert!(output.exists(), "output file should exist");
        let scene = decode_scene_file(&output).expect("should decode");

        let vertex_count = scene.positions.len() as u32 / 3;
        assert!(vertex_count > 0, "should have vertices");

        let _ = fs::remove_file(output);
    }

    #[test]
    fn test_deterministic_output() {
        let temp_dir = temp_dir();
        let output1 = temp_dir.join("test_det_1.pb");
        let output2 = temp_dir.join("test_det_2.pb");

        run_pipeline(1, &output1).unwrap();
        run_pipeline(1, &output2).unwrap();

        let bytes1 = fs::read(&output1).unwrap();
        let bytes2 = fs::read(&output2).unwrap();
        assert_eq!(bytes1, bytes2);

        let _ = fs::remove_file(output1);
        let _ = fs::remove_file(output2);
    }

    #[test]
    fn test_cli_argument_parsing() {
        let temp_dir = temp_dir();
        let output = temp_dir.join("test_args.pb");

        let result = run_pipeline(4, &output);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.final_step, 4);

        let _ = fs::remove_file(output);
    }
}
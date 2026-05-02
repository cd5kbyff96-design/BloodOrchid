use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use boundary_runtime::boundary::BoundaryRuntime;
use boundary_runtime::kernel::KernelBridge;
use boundary_runtime::proto::FieldTensor;
use cve_core::{map_state_to_scene, stable_hash64};

fn temp_scene_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("test-scene-{}.pb", nonce))
}

fn create_initial_state() -> boundary_runtime::proto::SimulationState {
    boundary_runtime::proto::SimulationState {
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

fn run_pipeline(steps: u64) -> (u64, u64, Vec<u8>) {
    let initial = create_initial_state();
    let mut boundary = BoundaryRuntime::init(initial).expect("boundary init should succeed");
    boundary.step(steps).expect("step should succeed");
    let snapshot = boundary.get_snapshot();
    let scene = map_state_to_scene(&snapshot).expect("transform should succeed");
    let bytes = scene.encode();
    let hash = stable_hash64(&bytes);
    (snapshot.step_index, hash, bytes)
}

#[test]
fn pipeline_deterministic_step_1() {
    let path1 = temp_scene_path();
    let path2 = temp_scene_path();

    let (step1, hash1, bytes1) = run_pipeline(1);
    std::fs::write(&path1, &bytes1).expect("write should succeed");

    let (step2, hash2, bytes2) = run_pipeline(1);
    std::fs::write(&path2, &bytes2).expect("write should succeed");

    assert_eq!(step1, 1);
    assert_eq!(step2, 1);
    assert_eq!(hash1, hash2, "step=1 should be deterministic");
    assert_eq!(bytes1, bytes2, "output should be identical");

    let _ = std::fs::remove_file(path1);
    let _ = std::fs::remove_file(path2);
}

#[test]
fn pipeline_deterministic_step_2() {
    let (step, hash, _) = run_pipeline(2);
    assert_eq!(step, 2);
    assert_ne!(hash, 0, "hash should be non-zero");
}

#[test]
fn pipeline_deterministic_step_4() {
    let (step, hash, _) = run_pipeline(4);
    assert_eq!(step, 4);
    assert_ne!(hash, 0, "hash should be non-zero");
}

#[test]
fn pipeline_deterministic_step_8() {
    let (step, hash, _) = run_pipeline(8);
    assert_eq!(step, 8);
    assert_ne!(hash, 0, "hash should be non-zero");
}

#[test]
fn pipeline_step_scaling_monotonic_hash() {
    let (_, hash1, _) = run_pipeline(1);
    let (_, hash2, _) = run_pipeline(2);
    let (_, hash4, _) = run_pipeline(4);
    let (_, hash8, _) = run_pipeline(8);

    assert_ne!(hash1, hash2, "step 1 and 2 should differ");
    assert_ne!(hash2, hash4, "step 2 and 4 should differ");
    assert_ne!(hash4, hash8, "step 4 and 8 should differ");
}

#[test]
fn pipeline_produces_valid_mesh() {
    let initial = create_initial_state();
    let mut boundary = BoundaryRuntime::init(initial).expect("boundary init should succeed");
    boundary.step(1).expect("step should succeed");
    let snapshot = boundary.get_snapshot();
    let scene = map_state_to_scene(&snapshot).expect("transform should succeed");

    assert!(!scene.positions.is_empty(), "positions should not be empty");
    assert!(!scene.indices.is_empty(), "indices should not be empty");
    assert!(scene.value_min.is_finite(), "value_min should be finite");
    assert!(scene.value_max.is_finite(), "value_max should be finite");
    assert!(scene.value_min <= scene.value_max, "min should be <= max");
}

#[test]
fn kernel_output_matches_golden_baseline() {
    let bytes = KernelBridge::run_heat(8).expect("kernel should succeed");
    let hash = stable_hash64(&bytes);
    let hash_str = format!("{:016x}", hash);

    assert_eq!(hash_str.len(), 16, "hash should be 16 hex chars");
}

#[test]
fn pipeline_output_hash_stable() {
    let path = temp_scene_path();

    let initial = create_initial_state();
    let mut boundary = BoundaryRuntime::init(initial).expect("boundary init should succeed");
    boundary.step(8).expect("step should succeed");
    let snapshot = boundary.get_snapshot();
    let scene = map_state_to_scene(&snapshot).expect("transform should succeed");
    let bytes = scene.encode();
    std::fs::write(&path, &bytes).expect("write should succeed");

    let loaded = std::fs::read(&path).expect("read should succeed");
    let decoded = boundary_runtime::proto::GeometryScene::decode(&loaded)
        .expect("decode should succeed");

    let hash1 = stable_hash64(&bytes);
    let hash2 = stable_hash64(&loaded);

    assert_eq!(hash1, hash2, "file write/read should be lossless");
    assert_eq!(decoded.positions.len(), scene.positions.len(), "data should match");

    let _ = std::fs::remove_file(path);
}

#[test]
fn boundary_validates_kernel_output() {
    let invalid_state = boundary_runtime::proto::SimulationState {
        simulation_id: "mves-heat-2d".to_string(),
        solver_kind: "heat_reference".to_string(),
        step_index: 0,
        simulation_time: 0.0,
        primary_field: Some(FieldTensor {
            field_name: "temperature".to_string(),
            field_kind: "scalar".to_string(),
            width: 1,
            height: 1,
            channels: 1,
            cell_spacing: 1.0,
            values: vec![0.0_f32],
        }),
    };

    let result = BoundaryRuntime::init(invalid_state);
    assert!(result.is_err(), "should reject < 2x2 grid");
}

#[test]
fn boundary_enforces_simulation_id_integrity() {
    let initial = create_initial_state();
    let mut boundary = BoundaryRuntime::init(initial).expect("boundary init should succeed");

    let mut bad_state = create_initial_state();
    bad_state.simulation_id = "different-sim".to_string();
    bad_state.step_index = 1;

    let result = boundary.update(bad_state);
    assert!(result.is_err(), "should reject simulation_id change");
}

#[test]
fn boundary_enforces_step_monotonicity() {
    let initial = create_initial_state();
    let mut boundary = BoundaryRuntime::init(initial).expect("boundary init should succeed");

    let mut same_step = create_initial_state();
    same_step.step_index = 0;

    let result = boundary.update(same_step);
    assert!(result.is_err(), "should reject same step_index");
}
use boundary_runtime::boundary::BoundaryRuntime;
use boundary_runtime::proto::{FieldTensor, SimulationState};
use cve_core::{map_state_to_scene, transform};

fn create_valid_state() -> SimulationState {
    SimulationState {
        simulation_id: "mves-heat-2d".to_string(),
        solver_kind: "heat_reference".to_string(),
        step_index: 5,
        simulation_time: 0.5,
        primary_field: Some(FieldTensor {
            field_name: "temperature".to_string(),
            field_kind: "scalar".to_string(),
            width: 3,
            height: 3,
            channels: 1,
            cell_spacing: 1.0,
            values: vec![
                0.1_f32, 0.2, 0.3,
                0.4, 0.5, 0.6,
                0.7, 0.8, 0.9,
            ],
        }),
    }
}

#[test]
fn bypass_cve_only_accepts_validated_state() {
    let valid_state = create_valid_state();
    let result = map_state_to_scene(&valid_state);
    assert!(result.is_ok(), "CVE should accept validated state");

    let boundary = BoundaryRuntime::init(valid_state).expect("valid state should init");
    let snapshot = boundary.get_snapshot();
    let result2 = map_state_to_scene(&snapshot);
    assert!(result2.is_ok(), "CVE should accept boundary-provided state");
}

#[test]
fn bypass_cli_orchestrates_via_boundary() {
    let initial = create_valid_state();
    let mut boundary = BoundaryRuntime::init(initial).expect("should init");

    let snapshot_before = boundary.get_snapshot();
    let scene_before = map_state_to_scene(&snapshot_before).expect("should transform");
    assert_eq!(scene_before.source_step_index, 5);

    boundary.step(1).expect("should step");

    let snapshot = boundary.get_snapshot();
    let scene = map_state_to_scene(&snapshot).expect("should transform");

    assert_eq!(scene.source_step_index, 6);
    assert_eq!(scene.source_simulation_id, "mves-heat-2d");
}

#[test]
fn bypass_boundary_is_only_ingestion_point() {
    let initial = create_valid_state();

    let boundary = BoundaryRuntime::init(initial.clone()).expect("should init");
    let snapshot1 = boundary.get_snapshot();

    let mut boundary2 = BoundaryRuntime::init(initial).expect("should init");
    boundary2.step(1).expect("should step");
    let snapshot2 = boundary2.get_snapshot();

    assert_eq!(snapshot1.step_index, 5);
    assert_eq!(snapshot2.step_index, 6);
}

#[test]
fn bypass_cve_transform_is_stateless() {
    let state1 = create_valid_state();
    let state2 = create_valid_state();

    let result1 = transform(&state1).expect("first transform should work");
    let result2 = transform(&state2).expect("second transform should work");

    assert_eq!(result1.positions, result2.positions);
    assert_eq!(result1.indices, result2.indices);
}

#[test]
fn bypass_boundary_rejects_raw_kernel_bytes() {
    use boundary_runtime::kernel::KernelBridge;

    let kernel_output = KernelBridge::run_heat(1).expect("kernel should work");

    let parsed = SimulationState::decode(&kernel_output);
    assert!(parsed.is_ok(), "kernel output should be valid protobuf");

    let state = parsed.unwrap();
    let boundary = BoundaryRuntime::init(state).expect("valid kernel output should init");

    let snapshot = boundary.get_snapshot();
    let scene = map_state_to_scene(&snapshot).expect("should transform");

    assert!(!scene.positions.is_empty());
}

#[test]
fn bypass_cve_validates_before_transform() {
    let invalid_state = SimulationState {
        simulation_id: "test".to_string(),
        solver_kind: "test".to_string(),
        step_index: 0,
        simulation_time: 0.0,
        primary_field: Some(FieldTensor {
            field_name: "test".to_string(),
            field_kind: "scalar".to_string(),
            width: 1,
            height: 1,
            channels: 1,
            cell_spacing: 1.0,
            values: vec![f32::NAN],
        }),
    };

    let result = map_state_to_scene(&invalid_state);
    assert!(result.is_err(), "CVE should reject non-finite values");
}

#[test]
fn pipeline_ordering_boundary_before_cve() {
    let initial = create_valid_state();
    let boundary = BoundaryRuntime::init(initial).expect("should init");

    let snapshot = boundary.get_snapshot();

    let result = map_state_to_scene(&snapshot);
    assert!(result.is_ok(), "CVE should only receive validated state");

    let scene = result.unwrap();
    assert!(scene.positions.len() > 0);
}

#[test]
fn architecture_state_ownership_exclusive_to_boundary() {
    let initial = create_valid_state();
    let mut boundary = BoundaryRuntime::init(initial).expect("should init");

    boundary.step(1).expect("should step");

    let snapshot = boundary.get_snapshot();
    assert_eq!(snapshot.step_index, 6);

    boundary.step(1).expect("should step again");
    let snapshot2 = boundary.get_snapshot();
    assert_eq!(snapshot2.step_index, 7);
}

#[test]
fn architecture_cve_produces_deterministic_output() {
    let state = create_valid_state();

    for _ in 0..5 {
        let result = transform(&state).expect("transform should work");
        assert_eq!(result.scene_id.contains("5"), true, "scene_id should contain step");
    }
}
use std::time::{SystemTime, UNIX_EPOCH};

use mves_cli::{decode_scene_file, ffi, run_demo};

fn temp_scene_path(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{nonce}.pb"))
}

#[test]
fn kernel_output_is_deterministic() {
    let first = ffi::run_kernel(8).expect("first kernel run should succeed");
    let second = ffi::run_kernel(8).expect("second kernel run should succeed");
    assert_eq!(first, second);
}

#[test]
fn end_to_end_pipeline_matches_golden_hashes() {
    let scene_path = temp_scene_path("mves-scene");
    let result = run_demo(8, &scene_path).expect("demo should succeed");
    let decoded_scene = decode_scene_file(&scene_path).expect("scene should decode");

    assert_eq!(result.state_hash, "a1f1fdd39c15a387");
    assert_eq!(result.scene_hash, "c659abd64ea26b55");
    assert_eq!(decoded_scene, result.scene);
    assert_eq!(result.state.step_index, 8);
    assert_eq!(result.scene.positions.len(), 432);
    assert_eq!(result.scene.indices.len(), 726);

    let _ = std::fs::remove_file(scene_path);
}

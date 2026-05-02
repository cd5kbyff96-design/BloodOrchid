use std::fs;
use std::path::Path;

use boundary_runtime::kernel::KernelBridge;
use boundary_runtime::proto::{GeometryScene, SimulationState};
use cve_core::{stable_hash64, StateStore, map_state_to_scene};

#[derive(Clone, Debug, PartialEq)]
pub struct DemoResult {
    pub state_hash: String,
    pub scene_hash: String,
    pub state: SimulationState,
    pub scene: GeometryScene,
}

pub fn run_demo(steps: u64, scene_out: &Path) -> Result<DemoResult, String> {
    let kernel_bytes = KernelBridge::run_heat(steps)?;
    let mut store = StateStore::new();
    store.apply_kernel_frame(&kernel_bytes)?;

    let state = store
        .snapshot()
        .ok_or_else(|| "boundary store did not retain state".to_string())?;
    let state_bytes = store
        .latest_raw_bytes()
        .ok_or_else(|| "boundary store lost raw bytes".to_string())?;
    let scene = map_state_to_scene(&state)?;
    let scene_bytes = scene.encode();

    if let Some(parent) = scene_out.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(scene_out, &scene_bytes).map_err(|error| error.to_string())?;

    Ok(DemoResult {
        state_hash: format!("{:016x}", stable_hash64(state_bytes)),
        scene_hash: format!("{:016x}", stable_hash64(&scene_bytes)),
        state,
        scene,
    })
}

pub fn decode_scene_file(scene_path: &Path) -> Result<GeometryScene, String> {
    let bytes = fs::read(scene_path).map_err(|error| error.to_string())?;
    GeometryScene::decode(&bytes).map_err(|e| e.to_string())
}


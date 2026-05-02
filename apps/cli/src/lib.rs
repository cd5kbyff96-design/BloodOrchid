use std::fs;
use std::path::Path;

use boundary_runtime::boundary::BoundaryRuntime;
use boundary_runtime::kernel::KernelBridge;
use boundary_runtime::proto::{GeometryScene, SimulationState};
use cve_core::{stable_hash64, map_state_to_scene};

#[derive(Clone, Debug, PartialEq)]
pub struct DemoResult {
    pub state_hash: String,
    pub scene_hash: String,
    pub state: SimulationState,
    pub scene: GeometryScene,
}

pub fn run_demo(steps: u64, scene_out: &Path) -> Result<DemoResult, String> {
    let kernel_bytes = KernelBridge::run_heat(steps)?;
    let state = SimulationState::decode(&kernel_bytes)
        .map_err(|e| format!("decode failed: {}", e))?;

    let runtime = BoundaryRuntime::init(state)
        .map_err(|e| format!("boundary init failed: {}", e))?;

    let snapshot = runtime.get_snapshot();
    let state_bytes = snapshot.encode();
    let scene = map_state_to_scene(&snapshot)?;
    let scene_bytes = scene.encode();

    if let Some(parent) = scene_out.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(scene_out, &scene_bytes).map_err(|error| error.to_string())?;

    Ok(DemoResult {
        state_hash: format!("{:016x}", stable_hash64(&state_bytes)),
        scene_hash: format!("{:016x}", stable_hash64(&scene_bytes)),
        state: (*snapshot).clone(),
        scene,
    })
}

pub fn decode_scene_file(scene_path: &Path) -> Result<GeometryScene, String> {
    let bytes = fs::read(scene_path).map_err(|error| error.to_string())?;
    GeometryScene::decode(&bytes).map_err(|e| e.to_string())
}
use std::fs;
use std::path::Path;

use boundary_runtime::proto::{GeometryScene, SimulationState};
use boundary_runtime::{stable_hash64, StateStore};
use cve_core::map_state_to_scene;

pub mod ffi {
    use std::slice;

    unsafe extern "C" {
        fn mves_kernel_run_heat(
            steps: u64,
            out_ptr: *mut *const u8,
            out_len: *mut usize,
        ) -> bool;
        fn mves_kernel_free_buffer(ptr: *const u8, len: usize);
    }

    pub fn run_kernel(steps: u64) -> Result<Vec<u8>, String> {
        let mut ptr = std::ptr::null();
        let mut len = 0usize;
        let success = unsafe { mves_kernel_run_heat(steps, &mut ptr, &mut len) };
        if !success {
            return Err("kernel execution failed".into());
        }
        if ptr.is_null() || len == 0 {
            return Err("kernel returned an empty frame".into());
        }

        let bytes = unsafe { slice::from_raw_parts(ptr, len).to_vec() };
        unsafe { mves_kernel_free_buffer(ptr, len) };
        Ok(bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoResult {
    pub state_hash: String,
    pub scene_hash: String,
    pub state: SimulationState,
    pub scene: GeometryScene,
}

pub fn run_demo(steps: u64, scene_out: &Path) -> Result<DemoResult, String> {
    let kernel_bytes = ffi::run_kernel(steps)?;
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
    GeometryScene::decode(&bytes)
}


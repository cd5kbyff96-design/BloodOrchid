pub mod proto;

use std::sync::Arc;

use boundary_runtime::{SimulationState, GeometryScene};

pub fn map_state_to_scene(state: &SimulationState) -> Result<GeometryScene, String> {
    let field = state.primary_field
        .as_ref()
        .ok_or("primary_field is missing")?;
    let width = field.width as usize;
    let height = field.height as usize;
    let channels = field.channels as usize;

    let mut positions = Vec::with_capacity(width * height * channels * 3);
    for c in 0..channels {
        for y in 0..height {
            for x in 0..width {
                let idx = c * width * height + y * width + x;
                let value = field.values.get(idx).copied().unwrap_or(0.0);
                positions.push(x as f32 * field.cell_spacing);
                positions.push(y as f32 * field.cell_spacing);
                positions.push(value);
            }
        }
    }

    let mut indices = Vec::new();
    for y in 0..height.saturating_sub(1) {
        for x in 0..width.saturating_sub(1) {
            let top_left = (y * width + x) as u32;
            let top_right = top_left + 1;
            let bottom_left = top_left + width as u32;
            let bottom_right = bottom_left + 1;
            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);
            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    let value_min = field.values.iter().cloned().fold(f32::INFINITY, f32::min);
    let value_max = field.values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    Ok(GeometryScene {
        scene_id: format!("scene-{}-{}", state.simulation_id, state.step_index),
        source_simulation_id: state.simulation_id.clone(),
        source_step_index: state.step_index,
        positions,
        indices,
        value_min,
        value_max,
    })
}

#[derive(Clone, Debug)]
pub struct StateCache {
    state: Option<Arc<SimulationState>>,
}

impl StateCache {
    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn accept(&mut self, snapshot: Arc<SimulationState>) {
        self.state = Some(snapshot);
    }

    pub fn get(&self) -> Option<Arc<SimulationState>> {
        self.state.clone()
    }
}

impl Default for StateCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn stable_hash64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::FieldTensor;

    fn sample_state(step_index: u64) -> SimulationState {
        SimulationState {
            simulation_id: "mves-heat-2d".into(),
            solver_kind: "heat_reference".into(),
            step_index,
            simulation_time: step_index as f64 * 0.1,
            primary_field: Some(FieldTensor {
                field_name: "temperature".into(),
                field_kind: "scalar".into(),
                width: 3,
                height: 2,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![0.0_f32, 0.5, 1.0, 0.25, 0.75, 0.125],
            }),
        }
    }

    #[test]
    fn protobuf_roundtrip_preserves_simulation_state() {
        let state = sample_state(4);
        let bytes = state.encode();
        let decoded = SimulationState::decode(&bytes).expect("decode should succeed");
        assert_eq!(decoded, state);
    }

    #[test]
    fn state_cache_accepts_from_boundary() {
        let first = Arc::new(sample_state(2));
        let second = Arc::new(sample_state(3));

        let mut cache = StateCache::new();
        cache.accept(first);
        let first_snapshot = cache.get().expect("snapshot should exist");

        cache.accept(second);
        let second_snapshot = cache.get().expect("snapshot should exist");

        assert_eq!(first_snapshot.step_index, 2);
        assert_eq!(second_snapshot.step_index, 3);
    }

    #[test]
    fn state_cache_is_empty_before_accept() {
        let cache = StateCache::new();
        assert!(cache.get().is_none());
    }
}
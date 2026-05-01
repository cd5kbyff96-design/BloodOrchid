pub mod proto;

use boundary_runtime::{SimulationState, GeometryScene};

pub fn map_state_to_scene(state: &SimulationState) -> Result<GeometryScene, String> {
    let field = &state.primary_field;
    let width = field.width as usize;
    let height = field.height as usize;
    let channels = field.channels as usize;

    let mut positions = Vec::with_capacity(width * height * channels * 3);
    for c in 0..channels {
        for y in 0..height {
            for x in 0..width {
                let idx = c * width * height + y * width + x;
                let value = field.values.get(idx).copied().unwrap_or(0.0);
                positions.push(x as f64 * field.cell_spacing);
                positions.push(y as f64 * field.cell_spacing);
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

    let values: Vec<f64> = field.values.iter().cloned().collect();
    let value_min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let value_max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

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

#[derive(Clone, Debug, PartialEq)]
pub struct StoredState {
    pub state: SimulationState,
    pub raw_bytes: Vec<u8>,
}

#[derive(Default, Debug)]
pub struct StateStore {
    latest: Option<StoredState>,
}

impl StateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_kernel_frame(&mut self, bytes: &[u8]) -> Result<(), String> {
        let state = SimulationState::decode(bytes)?;
        validate_state(&state)?;
        self.latest = Some(StoredState {
            state,
            raw_bytes: bytes.to_vec(),
        });
        Ok(())
    }

    pub fn snapshot(&self) -> Option<SimulationState> {
        self.latest.as_ref().map(|stored| stored.state.clone())
    }

    pub fn latest_raw_bytes(&self) -> Option<&[u8]> {
        self.latest.as_ref().map(|stored| stored.raw_bytes.as_slice())
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

fn validate_state(state: &SimulationState) -> Result<(), String> {
    if state.simulation_id.trim().is_empty() {
        return Err("simulation_id must not be empty".into());
    }
    if state.solver_kind.trim().is_empty() {
        return Err("solver_kind must not be empty".into());
    }

    let field = &state.primary_field;
    if field.field_name.trim().is_empty() {
        return Err("primary_field.field_name must not be empty".into());
    }
    if field.field_kind.trim().is_empty() {
        return Err("primary_field.field_kind must not be empty".into());
    }
    if field.width < 2 || field.height < 2 {
        return Err("field grid must be at least 2x2".into());
    }
    if field.channels == 0 {
        return Err("field must have at least one channel".into());
    }
    if !field.cell_spacing.is_finite() || field.cell_spacing <= 0.0 {
        return Err("cell_spacing must be finite and positive".into());
    }
    let expected_len = field.width as usize * field.height as usize * field.channels as usize;
    if field.values.len() != expected_len {
        return Err(format!(
            "field value count mismatch: expected {expected_len}, got {}",
            field.values.len()
        ));
    }
    if field.values.iter().any(|value| !value.is_finite()) {
        return Err("field contains non-finite values".into());
    }
    if !state.simulation_time.is_finite() || state.simulation_time < 0.0 {
        return Err("simulation_time must be finite and non-negative".into());
    }
    Ok(())
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
            tick: 0,
            domain: "test".into(),
            primary_field: FieldTensor {
                field_name: "temperature".into(),
                field_kind: "scalar".into(),
                width: 3,
                height: 2,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![0.0, 0.5, 1.0, 0.25, 0.75, 0.125],
            },
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
    fn state_store_snapshot_is_owned_and_latest() {
        let first = sample_state(2);
        let second = sample_state(3);

        let mut store = StateStore::new();
        store
            .apply_kernel_frame(&first.encode())
            .expect("first state should apply");
        let first_snapshot = store.snapshot().expect("snapshot should exist");

        store
            .apply_kernel_frame(&second.encode())
            .expect("second state should apply");
        let second_snapshot = store.snapshot().expect("snapshot should exist");

        assert_eq!(first_snapshot.step_index, 2);
        assert_eq!(second_snapshot.step_index, 3);
    }

    #[test]
    fn state_store_rejects_invalid_shapes() {
        let mut invalid = sample_state(1);
        invalid.primary_field.values.pop();

        let mut store = StateStore::new();
        let error = store
            .apply_kernel_frame(&invalid.encode())
            .expect_err("shape mismatch should fail");

        assert!(error.contains("field value count mismatch"));
        assert!(store.snapshot().is_none());
    }
}


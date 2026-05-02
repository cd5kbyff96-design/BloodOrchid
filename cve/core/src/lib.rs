pub mod proto;

use boundary_runtime::proto::{FieldTensor, GeometryScene};

pub fn calculate_mesh_dimensions(width: u32, height: u32) -> (u32, u32) {
    let vertex_count = width * height;
    let index_count = (width - 1) * (height - 1) * 6;
    (vertex_count, index_count)
}

pub fn generate_vertices(field: &FieldTensor) -> Vec<f32> {
    let width = field.width as usize;
    let height = field.height as usize;
    let spacing = field.cell_spacing;

    let mut vertices = Vec::with_capacity(width * height * 3);

    for row in 0..height {
        for col in 0..width {
            let index = row * width + col;
            let z = field.values[index];

            let x = col as f32 * spacing;
            let y = row as f32 * spacing;

            vertices.push(x);
            vertices.push(y);
            vertices.push(z);
        }
    }

    vertices
}

pub fn generate_indices(width: u32, height: u32) -> Vec<u32> {
    let w = width as usize;
    let h = height as usize;

    let mut indices = Vec::with_capacity((w - 1) * (h - 1) * 6);

    for row in 0..(h - 1) {
        for col in 0..(w - 1) {
            let top_left = (row * w + col) as u32;
            let top_right = top_left + 1;
            let bottom_left = ((row + 1) * w + col) as u32;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);
            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    indices
}

pub fn compute_extents(values: &[f32]) -> (f32, f32, f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    let mut i = 0;
    while i < values.len() {
        let x = values[i];
        let y = values[i + 1];
        let z = values[i + 2];

        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
        if z < min_z {
            min_z = z;
        }
        if z > max_z {
            max_z = z;
        }

        i += 3;
    }

    (min_x, max_x, min_y, max_y, min_z, max_z)
}

pub fn transform(state: &boundary_runtime::proto::SimulationState) -> Result<GeometryScene, String> {
    map_state_to_scene(state)
}

pub fn map_state_to_scene(state: &boundary_runtime::proto::SimulationState) -> Result<GeometryScene, String> {
    let field = state.primary_field.as_ref().ok_or("primary_field is missing")?;

    if field.width < 2 || field.height < 2 {
        return Err("field dimensions must be at least 2x2".into());
    }

    if field.channels != 1 {
        return Err("CVE core only supports single-channel scalar fields".into());
    }

    let expected_values = (field.width * field.height) as usize;
    if field.values.len() != expected_values {
        return Err("field values do not match grid dimensions".into());
    }

    for val in &field.values {
        if !val.is_finite() {
            return Err("field contains non-finite values".into());
        }
    }

    let _ = calculate_mesh_dimensions(field.width, field.height);
    let vertices = generate_vertices(field);
    let indices = generate_indices(field.width, field.height);

    let (_min_x, _max_x, _min_y, _max_y, value_min, value_max) = compute_extents(&vertices);

    let (expected_vertices, expected_indices) = calculate_mesh_dimensions(field.width, field.height);
    let actual_vertices = vertices.len() as u32 / 3;
    let actual_indices = indices.len() as u32;

    if actual_vertices != expected_vertices || actual_indices != expected_indices {
        return Err("mesh generation produced unexpected dimensions".into());
    }

    let scene_id = format!("scene-{}-{}", state.simulation_id, state.step_index);

    Ok(GeometryScene {
        scene_id,
        source_simulation_id: state.simulation_id.clone(),
        source_step_index: state.step_index,
        positions: vertices,
        indices,
        value_min,
        value_max,
    })
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
    use boundary_runtime::proto::FieldTensor;

    fn sample_state(step_index: u64) -> boundary_runtime::proto::SimulationState {
        boundary_runtime::proto::SimulationState {
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
    fn test_identity_transformation() {
        let state = sample_state(5);

        let result1 = transform(&state).expect("first transform should succeed");
        let result2 = transform(&state).expect("second transform should succeed");
        let result3 = transform(&state).expect("third transform should succeed");

        assert_eq!(result1.scene_id, result2.scene_id);
        assert_eq!(result2.scene_id, result3.scene_id);
        assert_eq!(result1.positions, result2.positions);
        assert_eq!(result2.positions, result3.positions);
        assert_eq!(result1.indices, result2.indices);
        assert_eq!(result2.indices, result3.indices);
        assert_eq!(result1.value_min, result2.value_min);
        assert_eq!(result1.value_max, result2.value_max);
    }

    #[test]
    fn test_mesh_boundary_alignment() {
        let state = sample_state(1);
        let result = transform(&state).expect("transform should succeed");

        let positions = &result.positions;
        let width = 3u32;
        let height = 2u32;
        let spacing = 1.0f32;

        let first_vertex_x = positions[0];
        let first_vertex_y = positions[1];
        assert_eq!(first_vertex_x, 0.0);
        assert_eq!(first_vertex_y, 0.0);

        let last_col_idx = (width - 1) as usize;
        let last_row_idx = (height - 1) as usize;
        let last_vertex_idx = last_row_idx * width as usize + last_col_idx;
        let last_x = positions[last_vertex_idx * 3];
        let last_y = positions[last_vertex_idx * 3 + 1];
        assert_eq!(last_x, (width - 1) as f32 * spacing);
        assert_eq!(last_y, (height - 1) as f32 * spacing);
    }

    #[test]
    fn test_index_range_safety() {
        let state = sample_state(1);
        let result = transform(&state).expect("transform should succeed");

        let vertex_count = 3u32 * 2u32;
        for &idx in &result.indices {
            assert!(idx < vertex_count, "index {} out of bounds for {} vertices", idx, vertex_count);
        }

        let cell_count = (3u32 - 1) * (2u32 - 1);
        let expected_triangles = cell_count * 2;
        let expected_indices = expected_triangles * 3;
        assert_eq!(result.indices.len() as u32, expected_indices);
    }

    #[test]
    fn test_monomorphic_scene_id() {
        let state1 = sample_state(1);
        let state2 = sample_state(2);

        let result1 = transform(&state1).expect("transform should succeed");
        let result2 = transform(&state2).expect("transform should succeed");

        assert_eq!(result1.source_step_index, 1);
        assert_eq!(result2.source_step_index, 2);

        assert!(result1.scene_id.contains("1"));
        assert!(result2.scene_id.contains("2"));
    }

    #[test]
    fn test_finite_output_guarantee() {
        let state = sample_state(1);
        let result = transform(&state).expect("transform should succeed");

        for val in &result.positions {
            assert!(val.is_finite(), "vertex position {} is not finite", val);
        }

        assert!(result.value_min.is_finite());
        assert!(result.value_max.is_finite());
        assert!(result.value_min <= result.value_max);
    }

    #[test]
    fn test_calculate_mesh_dimensions() {
        let (vertices, indices) = calculate_mesh_dimensions(4, 3);
        assert_eq!(vertices, 12);
        assert_eq!(indices, 36);

        let (vertices, indices) = calculate_mesh_dimensions(2, 2);
        assert_eq!(vertices, 4);
        assert_eq!(indices, 6);
    }

    #[test]
    fn test_generate_vertices() {
        let field = FieldTensor {
            field_name: "test".into(),
            field_kind: "scalar".into(),
            width: 2,
            height: 2,
            channels: 1,
            cell_spacing: 2.0,
            values: vec![1.0, 2.0, 3.0, 4.0],
        };

        let vertices = generate_vertices(&field);

        assert_eq!(vertices.len(), 12);
        assert_eq!(vertices[0], 0.0);
        assert_eq!(vertices[1], 0.0);
        assert_eq!(vertices[2], 1.0);

        assert_eq!(vertices[3], 2.0);
        assert_eq!(vertices[4], 0.0);
        assert_eq!(vertices[5], 2.0);

        assert_eq!(vertices[6], 0.0);
        assert_eq!(vertices[7], 2.0);
        assert_eq!(vertices[8], 3.0);

        assert_eq!(vertices[9], 2.0);
        assert_eq!(vertices[10], 2.0);
        assert_eq!(vertices[11], 4.0);
    }

    #[test]
    fn test_generate_indices() {
        let indices = generate_indices(3, 3);

        assert_eq!(indices.len(), 24);

        assert_eq!(indices[0], 0);
        assert_eq!(indices[1], 3);
        assert_eq!(indices[2], 1);
        assert_eq!(indices[3], 1);
        assert_eq!(indices[4], 3);
        assert_eq!(indices[5], 4);

        assert_eq!(indices[6], 1);
        assert_eq!(indices[7], 4);
        assert_eq!(indices[8], 2);
    }

    #[test]
    fn test_compute_extents() {
        let values = vec![
            0.0, 0.0, 0.0,
            1.0, 0.0, 1.0,
            0.0, 1.0, 2.0,
            1.0, 1.0, 3.0,
        ];

        let (min_x, max_x, min_y, max_y, min_z, max_z) = compute_extents(&values);

        assert_eq!(min_x, 0.0);
        assert_eq!(max_x, 1.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_y, 1.0);
        assert_eq!(min_z, 0.0);
        assert_eq!(max_z, 3.0);
    }

    #[test]
    fn test_transform_minimum_field_size() {
        let state = boundary_runtime::proto::SimulationState {
            simulation_id: "test".into(),
            solver_kind: "test".into(),
            step_index: 0,
            simulation_time: 0.0,
            primary_field: Some(FieldTensor {
                field_name: "test".into(),
                field_kind: "scalar".into(),
                width: 2,
                height: 2,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![0.0, 1.0, 2.0, 3.0],
            }),
        };

        let result = transform(&state).expect("2x2 field should be valid");
        assert_eq!(result.positions.len(), 12);
        assert_eq!(result.indices.len(), 6);
    }

    #[test]
    fn test_transform_rejects_non_finite_values() {
        let state = boundary_runtime::proto::SimulationState {
            simulation_id: "test".into(),
            solver_kind: "test".into(),
            step_index: 0,
            simulation_time: 0.0,
            primary_field: Some(FieldTensor {
                field_name: "test".into(),
                field_kind: "scalar".into(),
                width: 2,
                height: 2,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![0.0, f32::NAN, 2.0, 3.0],
            }),
        };

        let result = transform(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_rejects_invalid_dimensions() {
        let state = boundary_runtime::proto::SimulationState {
            simulation_id: "test".into(),
            solver_kind: "test".into(),
            step_index: 0,
            simulation_time: 0.0,
            primary_field: Some(FieldTensor {
                field_name: "test".into(),
                field_kind: "scalar".into(),
                width: 1,
                height: 2,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![0.0, 1.0],
            }),
        };

        let result = transform(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_rejects_multi_channel() {
        let state = boundary_runtime::proto::SimulationState {
            simulation_id: "test".into(),
            solver_kind: "test".into(),
            step_index: 0,
            simulation_time: 0.0,
            primary_field: Some(FieldTensor {
                field_name: "test".into(),
                field_kind: "scalar".into(),
                width: 2,
                height: 2,
                channels: 2,
                cell_spacing: 1.0,
                values: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
            }),
        };

        let result = transform(&state);
        assert!(result.is_err());
    }
}
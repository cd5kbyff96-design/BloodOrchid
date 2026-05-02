use boundary_runtime::proto::{GeometryScene, SimulationState};
use cve_core::{calculate_mesh_dimensions, compute_extents, generate_indices, generate_vertices};

pub fn map_state_to_scene(state: &SimulationState) -> Result<GeometryScene, String> {
    let field = &state.primary_field;
    if field.channels != 1 {
        return Err("CVE core only supports single-channel scalar fields".into());
    }

    let width = field.width as usize;
    let height = field.height as usize;
    if field.values.len() != width * height {
        return Err("field values do not match grid dimensions".into());
    }

    let (expected_vertices, expected_indices) = calculate_mesh_dimensions(field.width, field.height);
    let vertices = generate_vertices(field);
    let indices = generate_indices(field.width, field.height);

    if vertices.len() as u32 != expected_vertices * 3 {
        return Err("vertex count mismatch".into());
    }
    if indices.len() as u32 != expected_indices {
        return Err("index count mismatch".into());
    }

    let (_min_x, _max_x, _min_y, _max_y, value_min, value_max) = compute_extents(&vertices);

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

#[cfg(test)]
mod tests {
    use super::*;
    use boundary_runtime::proto::FieldTensor;

    #[test]
    fn mapping_builds_expected_mesh_sizes() {
        let state = SimulationState {
            simulation_id: "sim".into(),
            solver_kind: "heat".into(),
            step_index: 5,
            simulation_time: 0.5,
            primary_field: FieldTensor {
                field_name: "temperature".into(),
                field_kind: "scalar".into(),
                width: 3,
                height: 3,
                channels: 1,
                cell_spacing: 2.0,
                values: vec![
                    0.0, 0.1, 0.2,
                    0.3, 0.4, 0.5,
                    0.6, 0.7, 0.8,
                ],
            },
        };

        let scene = map_state_to_scene(&state).expect("mapping should succeed");

        assert_eq!(scene.positions.len(), 27);
        assert_eq!(scene.indices.len(), 24);
        assert_eq!(scene.value_min, 0.0);
        assert_eq!(scene.value_max, 0.8);
    }

    #[test]
    fn mapping_uses_cve_core_functions() {
        let state = SimulationState {
            simulation_id: "test".into(),
            solver_kind: "test".into(),
            step_index: 1,
            simulation_time: 0.1,
            primary_field: FieldTensor {
                field_name: "test".into(),
                field_kind: "scalar".into(),
                width: 2,
                height: 2,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![1.0, 2.0, 3.0, 4.0],
            },
        };

        let scene = map_state_to_scene(&state).expect("mapping should succeed");

        assert_eq!(scene.positions.len(), 12);
        assert_eq!(scene.indices.len(), 6);

        assert_eq!(scene.positions[0], 0.0);
        assert_eq!(scene.positions[1], 0.0);
        assert_eq!(scene.positions[2], 1.0);

        assert_eq!(scene.value_min, 1.0);
        assert_eq!(scene.value_max, 4.0);
    }
}
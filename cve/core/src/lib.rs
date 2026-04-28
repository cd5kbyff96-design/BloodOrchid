use boundary_runtime::proto::{GeometryScene, SimulationState};

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

    let mut positions = Vec::with_capacity(width * height * 3);
    for row in 0..height {
        for col in 0..width {
            let index = row * width + col;
            positions.push(col as f32 * field.cell_spacing);
            positions.push(row as f32 * field.cell_spacing);
            positions.push(field.values[index]);
        }
    }

    let mut indices = Vec::with_capacity((width - 1) * (height - 1) * 6);
    for row in 0..(height - 1) {
        for col in 0..(width - 1) {
            let top_left = (row * width + col) as u32;
            let top_right = top_left + 1;
            let bottom_left = ((row + 1) * width + col) as u32;
            let bottom_right = bottom_left + 1;

            indices.extend_from_slice(&[
                top_left,
                bottom_left,
                top_right,
                top_right,
                bottom_left,
                bottom_right,
            ]);
        }
    }

    let (value_min, value_max) = min_max(&field.values);

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

fn min_max(values: &[f32]) -> (f32, f32) {
    let mut min = values[0];
    let mut max = values[0];
    for value in values.iter().copied().skip(1) {
        if value < min {
            min = value;
        }
        if value > max {
            max = value;
        }
    }
    (min, max)
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
}


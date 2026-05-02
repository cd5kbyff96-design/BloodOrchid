pub mod proto;
pub mod kernel;
pub mod boundary;
pub use proto::{SimulationState, FieldTensor, GeometryScene};

pub use vail_iris_contracts::invariants::{InvariantRequest, InvariantResponse};

pub const BOUNDARY_INVARIANTS_CONTRACT_PATH: &str =
    "contracts/boundary_invariants/contracts.proto";

pub fn evaluate_invariant_request(request: &InvariantRequest) -> InvariantResponse {
    match validate_simulation_state_payload(&request.simulation_state) {
        Ok(()) => InvariantResponse {
            accepted: true,
            violations: Vec::new(),
        },
        Err(msg) => InvariantResponse {
            accepted: false,
            violations: vec![msg],
        },
    }
}

pub fn validate_simulation_state_payload(payload: &[u8]) -> Result<(), String> {
    if payload.is_empty() {
        return Err("simulation_state is empty".to_string());
    }

    let state = SimulationState::decode(payload)
        .map_err(|e| format!("simulation_state decode failed: {}", e))?;

    if state.simulation_id.trim().is_empty() {
        return Err("simulation_id must not be empty".to_string());
    }
    if state.solver_kind.trim().is_empty() {
        return Err("solver_kind must not be empty".to_string());
    }

    let field = state.primary_field
        .as_ref()
        .ok_or("primary_field is missing")?;

    if field.width < 2 || field.height < 2 {
        return Err("primary_field grid must be at least 2x2".to_string());
    }
    if field.channels == 0 {
        return Err("primary_field channels must be >= 1".to_string());
    }
    if !field.cell_spacing.is_finite() || field.cell_spacing <= 0.0 {
        return Err("primary_field cell_spacing must be finite and positive".to_string());
    }

    let expected = field.width as usize * field.height as usize * field.channels as usize;
    if field.values.len() != expected {
        return Err(format!(
            "primary_field values length mismatch: expected {expected}, got {}",
            field.values.len()
        ));
    }

    if field.values.iter().any(|v: &f32| !v.is_finite()) {
        return Err("primary_field contains non-finite values".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::FieldTensor;

    fn valid_state_bytes() -> Vec<u8> {
        let state = SimulationState {
            simulation_id: "fixture-sim".to_string(),
            solver_kind: "heat_reference".to_string(),
            step_index: 8,
            simulation_time: 0.8,
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
        };
        state.encode()
    }

    #[test]
    fn accepts_valid_simulation_state_fixture() {
        let req = InvariantRequest {
            simulation_state: valid_state_bytes(),
        };
        let resp = evaluate_invariant_request(&req);
        assert!(resp.accepted);
        assert!(resp.violations.is_empty());
    }

    #[test]
    fn rejects_malformed_simulation_state_fixture() {
        let req = InvariantRequest {
            simulation_state: vec![0xff, 0x01, 0x00],
        };
        let resp = evaluate_invariant_request(&req);
        assert!(!resp.accepted);
        assert!(!resp.violations.is_empty());
    }
}
use std::sync::Arc;

use crate::kernel::KernelBridge;
use crate::proto::{FieldTensor, SimulationState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub reason: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValidationError: {}", self.reason)
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    pub reason: String,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExecutionError: {}", self.reason)
    }
}

impl std::error::Error for ExecutionError {}

pub struct BoundaryRuntime {
    state: Arc<SimulationState>,
    simulation_id: Arc<String>,
    solver_kind: Arc<String>,
}

impl BoundaryRuntime {
    pub fn init(initial_state: SimulationState) -> Result<Self, ValidationError> {
        validate(&initial_state)?;
        let simulation_id = initial_state.simulation_id.clone();
        let solver_kind = initial_state.solver_kind.clone();
        Ok(Self {
            state: Arc::new(initial_state),
            simulation_id: Arc::new(simulation_id),
            solver_kind: Arc::new(solver_kind),
        })
    }

    pub fn update(&mut self, new_state: SimulationState) -> Result<(), ValidationError> {
        validate(&new_state)?;

        if new_state.simulation_id.as_str() != self.simulation_id.as_str() {
            return Err(ValidationError {
                reason: "simulation_id mismatch: cannot switch simulation context".to_string(),
            });
        }

        if new_state.solver_kind.as_str() != self.solver_kind.as_str() {
            return Err(ValidationError {
                reason: "solver_kind mismatch: cannot change solver mid-simulation".to_string(),
            });
        }

        let current_step = self.state.step_index;
        if new_state.step_index <= current_step {
            return Err(ValidationError {
                reason: format!(
                    "step_index must strictly increase: current={}, proposed={}",
                    current_step, new_state.step_index
                ),
            });
        }

        if new_state.simulation_time <= self.state.simulation_time {
            return Err(ValidationError {
                reason: format!(
                    "simulation_time must increase: current={}, proposed={}",
                    self.state.simulation_time, new_state.simulation_time
                ),
            });
        }

        self.state = Arc::new(new_state);
        Ok(())
    }

    pub fn step(&mut self, count: u64) -> Result<Arc<SimulationState>, ExecutionError> {
        let current_step = self.state.step_index;
        let target_step = current_step + count;

        // Extract the current state vector
        let field = self.state.primary_field
            .as_ref()
            .ok_or_else(|| ExecutionError {
                reason: "primary_field is missing".to_string(),
            })?;

        // Call kernel to advance the state by 'count' steps
        let mut output_state: *mut f32 = std::ptr::null_mut();
        let mut output_size: usize = 0;
        let result = unsafe {
            kernel::ffi::mves_kernel_advance_state(
                field.values.as_ptr(),
                field.values.len(),
                count,
                &mut output_state,
                &mut output_size,
            )
        };

        if !result {
            return Err(ExecutionError {
                reason: "kernel advance state failed".to_string(),
            });
        }

        // Extract the new state vector
        let new_values = unsafe {
            std::slice::from_raw_parts(output_state, output_size).to_vec()
        };

        // Free the memory allocated by the kernel
        unsafe {
            kernel::ffi::mves_kernel_free_state(output_state);
        }

        // Create new state with updated metadata
        let mut new_state = (*self.state).clone();
        new_state.step_index = self.state.step_index + count;
        new_state.simulation_time = self.state.simulation_time + (count as f64 * 0.1); // Assuming dt=0.1
        new_state.primary_field = Some(FieldTensor {
            field_name: field.field_name.clone(),
            field_kind: field.field_kind.clone(),
            width: field.width,
            height: field.height,
            channels: field.channels,
            cell_spacing: field.cell_spacing,
            values: new_values,
        });
        self.state = Arc::new(new_state);
        Ok(self.state.clone())
    }

    pub fn get_snapshot(&self) -> Arc<SimulationState> {
        self.state.clone()
    }
}

pub fn validate(state: &SimulationState) -> Result<(), ValidationError> {
    if state.simulation_id.trim().is_empty() {
        return Err(ValidationError {
            reason: "simulation_id must not be empty".to_string(),
        });
    }

    if state.solver_kind.trim().is_empty() {
        return Err(ValidationError {
            reason: "solver_kind must not be empty".to_string(),
        });
    }

    let field = state.primary_field
        .as_ref()
        .ok_or_else(|| ValidationError {
            reason: "primary_field is missing".to_string(),
        })?;

    if field.width < 2 || field.height < 2 {
        return Err(ValidationError {
            reason: "primary_field grid must be at least 2x2".to_string(),
        });
    }

    if field.channels == 0 {
        return Err(ValidationError {
            reason: "primary_field channels must be >= 1".to_string(),
        });
    }

    if !field.cell_spacing.is_finite() || field.cell_spacing <= 0.0 {
        return Err(ValidationError {
            reason: "primary_field cell_spacing must be finite and positive".to_string(),
        });
    }

    let expected = field.width as usize * field.height as usize * field.channels as usize;
    if field.values.len() != expected {
        return Err(ValidationError {
            reason: format!(
                "primary_field values length mismatch: expected {}, got {}",
                expected,
                field.values.len()
            ),
        });
    }

    if field.values.iter().any(|v| !v.is_finite()) {
        return Err(ValidationError {
            reason: "primary_field contains non-finite values".to_string(),
        });
    }

    if !state.simulation_time.is_finite() || state.simulation_time < 0.0 {
        return Err(ValidationError {
            reason: "simulation_time must be finite and non-negative".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::FieldTensor;

    fn make_valid_state(step_index: u64) -> SimulationState {
        SimulationState {
            simulation_id: "mves-heat-2d".to_string(),
            solver_kind: "heat_reference".to_string(),
            step_index,
            simulation_time: step_index as f64 * 0.1,
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
        }
    }

    #[test]
    fn test_boundary_initialization_validation() {
        let valid_state = make_valid_state(0);
        let result = BoundaryRuntime::init(valid_state);
        assert!(result.is_ok());

        let empty_id_state = SimulationState {
            simulation_id: "".to_string(),
            ..make_valid_state(0)
        };
        let result = BoundaryRuntime::init(empty_id_state);
        assert!(result.is_err());

        let missing_field_state = SimulationState {
            primary_field: None,
            ..make_valid_state(0)
        };
        let result = BoundaryRuntime::init(missing_field_state);
        assert!(result.is_err());
    }

    #[test]
    fn test_boundary_update_monotonicity() {
        let mut runtime = BoundaryRuntime::init(make_valid_state(0)).unwrap();

        let same_step = make_valid_state(0);
        let result = runtime.update(same_step);
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("strictly increase"));

        let lower_step = make_valid_state(0);
        let result = runtime.update(lower_step);
        assert!(result.is_err());

        let valid_next = make_valid_state(5);
        let mut runtime = BoundaryRuntime::init(make_valid_state(0)).unwrap();
        let result = runtime.update(valid_next);
        assert!(result.is_ok());
    }

    #[test]
    fn test_boundary_identity_protection() {
        let mut runtime = BoundaryRuntime::init(make_valid_state(0)).unwrap();

        let mut different_id = make_valid_state(1);
        different_id.simulation_id = "different-sim".to_string();
        let result = runtime.update(different_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("simulation_id"));

        let mut different_solver = make_valid_state(1);
        different_solver.solver_kind = "different-solver".to_string();
        let result = runtime.update(different_solver);
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("solver_kind"));
    }

    #[test]
    fn test_snapshot_immutability_guarantee() {
        let runtime = BoundaryRuntime::init(make_valid_state(0)).unwrap();
        let snapshot1 = runtime.get_snapshot();
        let snapshot2 = runtime.get_snapshot();

        assert!(Arc::ptr_eq(&snapshot1, &snapshot2));
    }

    #[test]
    fn test_kernel_step_integration() {
        let mut runtime = BoundaryRuntime::init(make_valid_state(0)).unwrap();
        let result = runtime.step(4);
        assert!(result.is_ok(), "step failed: {:?}", result);
        let snapshot = result.unwrap();
        assert_eq!(snapshot.step_index, 4);
    }

    #[test]
    fn test_atomic_update_failure_recovery() {
        let mut runtime = BoundaryRuntime::init(make_valid_state(0)).unwrap();

        let invalid_next = SimulationState {
            step_index: 1,
            ..make_valid_state(0)
        };
        let update_result = runtime.update(invalid_next);
        assert!(update_result.is_err());

        let snapshot = runtime.get_snapshot();
        assert_eq!(snapshot.step_index, 0);
    }
}
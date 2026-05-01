use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationState {
    pub simulation_id: String,
    pub solver_kind: String,
    pub step_index: u64,
    pub simulation_time: f64,
    pub tick: u64,
    pub primary_field: FieldTensor,
    pub domain: String,
}

impl SimulationState {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("decode failed: {}", e))
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldTensor {
    pub field_name: String,
    pub field_kind: String,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub cell_spacing: f64,
    pub values: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometryScene {
    pub scene_id: String,
    pub source_simulation_id: String,
    pub source_step_index: u64,
    pub positions: Vec<f64>,
    pub indices: Vec<u32>,
    pub value_min: f64,
    pub value_max: f64,
}

impl GeometryScene {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("decode failed: {}", e))
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}
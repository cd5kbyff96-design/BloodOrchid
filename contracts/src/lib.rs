pub mod mves {
    include!(concat!(env!("OUT_DIR"), "/vailiris.mves.v1.rs"));
}

pub mod invariants {
    include!(concat!(env!("OUT_DIR"), "/veiliris.boundary_invariants.v1.rs"));
}

pub use mves::{FieldTensor, GeometryScene, SimulationState};
pub use invariants::{InvariantRequest, InvariantResponse};

impl SimulationState {
    pub fn decode(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        prost::Message::decode(bytes)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        prost::Message::encode(self, &mut buf).expect("encode should not fail");
        buf
    }
}

impl FieldTensor {
    pub fn values_f64(&self) -> Vec<f64> {
        self.values.iter().map(|&v| v as f64).collect()
    }
}

impl GeometryScene {
    pub fn decode(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        prost::Message::decode(bytes)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        prost::Message::encode(self, &mut buf).expect("encode should not fail");
        buf
    }
}

impl InvariantRequest {
    pub fn decode(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        prost::Message::decode(bytes)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        prost::Message::encode(self, &mut buf).expect("encode should not fail");
        buf
    }
}

impl InvariantResponse {
    pub fn decode(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        prost::Message::decode(bytes)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        prost::Message::encode(self, &mut buf).expect("encode should not fail");
        buf
    }
}
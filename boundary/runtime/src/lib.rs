pub mod proto;
pub use proto::{SimulationState, FieldTensor, GeometryScene};

pub const BOUNDARY_INVARIANTS_CONTRACT_PATH: &str =
    "contracts/boundary_invariants/contracts.proto";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvariantRequest {
    pub simulation_state: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvariantResponse {
    pub accepted: bool,
    pub violations: Vec<String>,
}

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
        .map_err(|e| format!("simulation_state decode failed: {e}"))?;

    if state.simulation_id.trim().is_empty() {
        return Err("simulation_id must not be empty".to_string());
    }
    if state.solver_kind.trim().is_empty() {
        return Err("solver_kind must not be empty".to_string());
    }

    let field = &state.primary_field;
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

    if field.values.iter().any(|v| !v.is_finite()) {
        return Err("primary_field contains non-finite values".to_string());
    }

    Ok(())
}

// Minimal protobuf encode/decode for:
// package veiliris.boundary_invariants.v1;
// message InvariantRequest { bytes simulation_state = 1; }
// message InvariantResponse { bool accepted = 1; repeated string violations = 2; }

impl InvariantRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encode_key(1, 2, &mut out);
        encode_varint(self.simulation_state.len() as u64, &mut out);
        out.extend_from_slice(&self.simulation_state);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = 0usize;
        let mut simulation_state = Vec::new();

        while cursor < bytes.len() {
            let key = decode_varint(bytes, &mut cursor)?;
            let field_number = (key >> 3) as u32;
            let wire_type = (key & 0x07) as u8;

            match field_number {
                1 => {
                    simulation_state = decode_bytes(bytes, &mut cursor, wire_type)?.to_vec();
                }
                _ => skip_field(bytes, &mut cursor, wire_type)?,
            }
        }

        Ok(Self { simulation_state })
    }
}

impl InvariantResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        encode_key(1, 0, &mut out);
        encode_varint(if self.accepted { 1 } else { 0 }, &mut out);

        for violation in &self.violations {
            encode_key(2, 2, &mut out);
            encode_varint(violation.len() as u64, &mut out);
            out.extend_from_slice(violation.as_bytes());
        }

        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = 0usize;
        let mut accepted = false;
        let mut violations = Vec::new();

        while cursor < bytes.len() {
            let key = decode_varint(bytes, &mut cursor)?;
            let field_number = (key >> 3) as u32;
            let wire_type = (key & 0x07) as u8;

            match field_number {
                1 => {
                    let v = decode_varint_with_wire(bytes, &mut cursor, wire_type)?;
                    accepted = v != 0;
                }
                2 => {
                    let s = decode_string(bytes, &mut cursor, wire_type)?;
                    violations.push(s);
                }
                _ => skip_field(bytes, &mut cursor, wire_type)?,
            }
        }

        Ok(Self { accepted, violations })
    }
}

fn encode_key(field_number: u32, wire_type: u8, out: &mut Vec<u8>) {
    encode_varint(((field_number as u64) << 3) | wire_type as u64, out);
}

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn decode_varint(bytes: &[u8], cursor: &mut usize) -> Result<u64, String> {
    let mut value = 0u64;
    let mut shift = 0u32;

    while *cursor < bytes.len() {
        let byte = bytes[*cursor];
        *cursor += 1;
        value |= ((byte & 0x7f) as u64) << shift;

        if (byte & 0x80) == 0 {
            return Ok(value);
        }

        shift += 7;
        if shift > 63 {
            return Err("varint too large".to_string());
        }
    }

    Err("unexpected EOF reading varint".to_string())
}

fn decode_varint_with_wire(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<u64, String> {
    if wire_type != 0 {
        return Err(format!("expected varint wire type 0, got {wire_type}"));
    }
    decode_varint(bytes, cursor)
}

fn decode_bytes<'a>(bytes: &'a [u8], cursor: &mut usize, wire_type: u8) -> Result<&'a [u8], String> {
    if wire_type != 2 {
        return Err(format!("expected bytes wire type 2, got {wire_type}"));
    }

    let len = decode_varint(bytes, cursor)? as usize;
    if *cursor + len > bytes.len() {
        return Err("unexpected EOF reading bytes field".to_string());
    }

    let slice = &bytes[*cursor..*cursor + len];
    *cursor += len;
    Ok(slice)
}

fn decode_string(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<String, String> {
    let raw = decode_bytes(bytes, cursor, wire_type)?;
    std::str::from_utf8(raw)
        .map(|s| s.to_string())
        .map_err(|e| format!("invalid utf8 string: {e}"))
}

fn skip_field(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<(), String> {
    match wire_type {
        0 => {
            let _ = decode_varint(bytes, cursor)?;
            Ok(())
        }
        1 => {
            if *cursor + 8 > bytes.len() {
                return Err("unexpected EOF skipping 64-bit field".to_string());
            }
            *cursor += 8;
            Ok(())
        }
        2 => {
            let len = decode_varint(bytes, cursor)? as usize;
            if *cursor + len > bytes.len() {
                return Err("unexpected EOF skipping length-delimited field".to_string());
            }
            *cursor += len;
            Ok(())
        }
        5 => {
            if *cursor + 4 > bytes.len() {
                return Err("unexpected EOF skipping 32-bit field".to_string());
            }
            *cursor += 4;
            Ok(())
        }
        _ => Err(format!("unsupported wire type {wire_type}")),
    }
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
            tick: 0,
            domain: "test".to_string(),
            primary_field: FieldTensor {
                field_name: "temperature".to_string(),
                field_kind: "scalar".to_string(),
                width: 3,
                height: 3,
                channels: 1,
                cell_spacing: 1.0,
                values: vec![
                    0.1, 0.2, 0.3,
                    0.4, 0.5, 0.6,
                    0.7, 0.8, 0.9,
                ],
            },
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

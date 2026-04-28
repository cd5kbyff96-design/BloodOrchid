#[derive(Clone, Debug, PartialEq)]
pub struct FieldTensor {
    pub field_name: String,
    pub field_kind: String,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub cell_spacing: f32,
    pub values: Vec<f32>,
}

impl Eq for FieldTensor {}

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationState {
    pub simulation_id: String,
    pub solver_kind: String,
    pub step_index: u64,
    pub simulation_time: f64,
    pub primary_field: FieldTensor,
}

impl Eq for SimulationState {}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryScene {
    pub scene_id: String,
    pub source_simulation_id: String,
    pub source_step_index: u64,
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub value_min: f32,
    pub value_max: f32,
}

impl Eq for GeometryScene {}

impl FieldTensor {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encode_string(1, &self.field_name, &mut out);
        encode_string(2, &self.field_kind, &mut out);
        encode_u32(3, self.width, &mut out);
        encode_u32(4, self.height, &mut out);
        encode_u32(5, self.channels, &mut out);
        encode_f32(6, self.cell_spacing, &mut out);
        encode_packed_f32(7, &self.values, &mut out);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut field_name = String::new();
        let mut field_kind = String::new();
        let mut width = 0;
        let mut height = 0;
        let mut channels = 0;
        let mut cell_spacing = 0.0f32;
        let mut values = Vec::new();
        let mut cursor = 0usize;

        while cursor < bytes.len() {
            let key = decode_varint(bytes, &mut cursor)?;
            let field_number = (key >> 3) as u32;
            let wire_type = (key & 0x07) as u8;
            match field_number {
                1 => field_name = decode_string(bytes, &mut cursor, wire_type)?,
                2 => field_kind = decode_string(bytes, &mut cursor, wire_type)?,
                3 => width = decode_u32(bytes, &mut cursor, wire_type)?,
                4 => height = decode_u32(bytes, &mut cursor, wire_type)?,
                5 => channels = decode_u32(bytes, &mut cursor, wire_type)?,
                6 => cell_spacing = decode_f32(bytes, &mut cursor, wire_type)?,
                7 => values = decode_packed_f32(bytes, &mut cursor, wire_type)?,
                _ => skip_field(bytes, &mut cursor, wire_type)?,
            }
        }

        Ok(Self {
            field_name,
            field_kind,
            width,
            height,
            channels,
            cell_spacing,
            values,
        })
    }
}

impl SimulationState {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encode_string(1, &self.simulation_id, &mut out);
        encode_string(2, &self.solver_kind, &mut out);
        encode_u64(3, self.step_index, &mut out);
        encode_f64(4, self.simulation_time, &mut out);
        encode_message(5, &self.primary_field.encode(), &mut out);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut simulation_id = String::new();
        let mut solver_kind = String::new();
        let mut step_index = 0u64;
        let mut simulation_time = 0.0f64;
        let mut primary_field = None;
        let mut cursor = 0usize;

        while cursor < bytes.len() {
            let key = decode_varint(bytes, &mut cursor)?;
            let field_number = (key >> 3) as u32;
            let wire_type = (key & 0x07) as u8;
            match field_number {
                1 => simulation_id = decode_string(bytes, &mut cursor, wire_type)?,
                2 => solver_kind = decode_string(bytes, &mut cursor, wire_type)?,
                3 => step_index = decode_u64(bytes, &mut cursor, wire_type)?,
                4 => simulation_time = decode_f64(bytes, &mut cursor, wire_type)?,
                5 => {
                    let nested = decode_bytes(bytes, &mut cursor, wire_type)?;
                    primary_field = Some(FieldTensor::decode(nested)?);
                }
                _ => skip_field(bytes, &mut cursor, wire_type)?,
            }
        }

        Ok(Self {
            simulation_id,
            solver_kind,
            step_index,
            simulation_time,
            primary_field: primary_field.ok_or("missing primary_field")?,
        })
    }
}

impl GeometryScene {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encode_string(1, &self.scene_id, &mut out);
        encode_string(2, &self.source_simulation_id, &mut out);
        encode_u64(3, self.source_step_index, &mut out);
        encode_packed_f32(4, &self.positions, &mut out);
        encode_packed_u32(5, &self.indices, &mut out);
        encode_f32(6, self.value_min, &mut out);
        encode_f32(7, self.value_max, &mut out);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut scene_id = String::new();
        let mut source_simulation_id = String::new();
        let mut source_step_index = 0u64;
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut value_min = 0.0f32;
        let mut value_max = 0.0f32;
        let mut cursor = 0usize;

        while cursor < bytes.len() {
            let key = decode_varint(bytes, &mut cursor)?;
            let field_number = (key >> 3) as u32;
            let wire_type = (key & 0x07) as u8;
            match field_number {
                1 => scene_id = decode_string(bytes, &mut cursor, wire_type)?,
                2 => source_simulation_id = decode_string(bytes, &mut cursor, wire_type)?,
                3 => source_step_index = decode_u64(bytes, &mut cursor, wire_type)?,
                4 => positions = decode_packed_f32(bytes, &mut cursor, wire_type)?,
                5 => indices = decode_packed_u32(bytes, &mut cursor, wire_type)?,
                6 => value_min = decode_f32(bytes, &mut cursor, wire_type)?,
                7 => value_max = decode_f32(bytes, &mut cursor, wire_type)?,
                _ => skip_field(bytes, &mut cursor, wire_type)?,
            }
        }

        Ok(Self {
            scene_id,
            source_simulation_id,
            source_step_index,
            positions,
            indices,
            value_min,
            value_max,
        })
    }
}

fn encode_key(field_number: u32, wire_type: u8, out: &mut Vec<u8>) {
    encode_varint(((field_number as u64) << 3) | u64::from(wire_type), out);
}

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn encode_u32(field_number: u32, value: u32, out: &mut Vec<u8>) {
    encode_key(field_number, 0, out);
    encode_varint(u64::from(value), out);
}

fn encode_u64(field_number: u32, value: u64, out: &mut Vec<u8>) {
    encode_key(field_number, 0, out);
    encode_varint(value, out);
}

fn encode_f32(field_number: u32, value: f32, out: &mut Vec<u8>) {
    encode_key(field_number, 5, out);
    out.extend_from_slice(&value.to_le_bytes());
}

fn encode_f64(field_number: u32, value: f64, out: &mut Vec<u8>) {
    encode_key(field_number, 1, out);
    out.extend_from_slice(&value.to_le_bytes());
}

fn encode_string(field_number: u32, value: &str, out: &mut Vec<u8>) {
    encode_key(field_number, 2, out);
    encode_varint(value.len() as u64, out);
    out.extend_from_slice(value.as_bytes());
}

fn encode_message(field_number: u32, value: &[u8], out: &mut Vec<u8>) {
    encode_key(field_number, 2, out);
    encode_varint(value.len() as u64, out);
    out.extend_from_slice(value);
}

fn encode_packed_f32(field_number: u32, values: &[f32], out: &mut Vec<u8>) {
    encode_key(field_number, 2, out);
    encode_varint((values.len() * 4) as u64, out);
    for value in values {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

fn encode_packed_u32(field_number: u32, values: &[u32], out: &mut Vec<u8>) {
    let mut packed = Vec::new();
    for value in values {
        encode_varint(u64::from(*value), &mut packed);
    }
    encode_key(field_number, 2, out);
    encode_varint(packed.len() as u64, out);
    out.extend_from_slice(&packed);
}

fn decode_varint(bytes: &[u8], cursor: &mut usize) -> Result<u64, String> {
    let mut value = 0u64;
    let mut shift = 0u32;

    while *cursor < bytes.len() {
        let byte = bytes[*cursor];
        *cursor += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift > 63 {
            return Err("varint overflow".into());
        }
    }

    Err("unexpected eof while decoding varint".into())
}

fn decode_u32(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<u32, String> {
    if wire_type != 0 {
        return Err(format!("expected varint wire type, got {wire_type}"));
    }
    decode_varint(bytes, cursor).map(|value| value as u32)
}

fn decode_u64(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<u64, String> {
    if wire_type != 0 {
        return Err(format!("expected varint wire type, got {wire_type}"));
    }
    decode_varint(bytes, cursor)
}

fn decode_f32(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<f32, String> {
    if wire_type != 5 {
        return Err(format!("expected 32-bit wire type, got {wire_type}"));
    }
    let chunk = read_exact(bytes, cursor, 4)?;
    let mut raw = [0u8; 4];
    raw.copy_from_slice(chunk);
    Ok(f32::from_le_bytes(raw))
}

fn decode_f64(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<f64, String> {
    if wire_type != 1 {
        return Err(format!("expected 64-bit wire type, got {wire_type}"));
    }
    let chunk = read_exact(bytes, cursor, 8)?;
    let mut raw = [0u8; 8];
    raw.copy_from_slice(chunk);
    Ok(f64::from_le_bytes(raw))
}

fn decode_string<'a>(bytes: &'a [u8], cursor: &mut usize, wire_type: u8) -> Result<String, String> {
    let value = decode_bytes(bytes, cursor, wire_type)?;
    std::str::from_utf8(value)
        .map(|text| text.to_string())
        .map_err(|error| error.to_string())
}

fn decode_bytes<'a>(bytes: &'a [u8], cursor: &mut usize, wire_type: u8) -> Result<&'a [u8], String> {
    if wire_type != 2 {
        return Err(format!("expected length-delimited wire type, got {wire_type}"));
    }
    let len = decode_varint(bytes, cursor)? as usize;
    read_exact(bytes, cursor, len)
}

fn decode_packed_f32(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<Vec<f32>, String> {
    let packed = decode_bytes(bytes, cursor, wire_type)?;
    if packed.len() % 4 != 0 {
        return Err("packed f32 payload length must be divisible by 4".into());
    }
    let mut values = Vec::with_capacity(packed.len() / 4);
    for chunk in packed.chunks_exact(4) {
        let mut raw = [0u8; 4];
        raw.copy_from_slice(chunk);
        values.push(f32::from_le_bytes(raw));
    }
    Ok(values)
}

fn decode_packed_u32(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<Vec<u32>, String> {
    let packed = decode_bytes(bytes, cursor, wire_type)?;
    let mut values = Vec::new();
    let mut inner = 0usize;
    while inner < packed.len() {
        values.push(decode_varint(packed, &mut inner)? as u32);
    }
    Ok(values)
}

fn skip_field(bytes: &[u8], cursor: &mut usize, wire_type: u8) -> Result<(), String> {
    match wire_type {
        0 => {
            let _ = decode_varint(bytes, cursor)?;
            Ok(())
        }
        1 => {
            let _ = read_exact(bytes, cursor, 8)?;
            Ok(())
        }
        2 => {
            let len = decode_varint(bytes, cursor)? as usize;
            let _ = read_exact(bytes, cursor, len)?;
            Ok(())
        }
        5 => {
            let _ = read_exact(bytes, cursor, 4)?;
            Ok(())
        }
        _ => Err(format!("unsupported wire type {wire_type}")),
    }
}

fn read_exact<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], String> {
    if bytes.len().saturating_sub(*cursor) < len {
        return Err("unexpected eof while reading field".into());
    }
    let start = *cursor;
    let end = start + len;
    *cursor = end;
    Ok(&bytes[start..end])
}

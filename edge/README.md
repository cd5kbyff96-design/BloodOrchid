# Edge - Rust Certified Edge Runtime

## Protocols
- **OPC-UA** - Primary SCADA abstraction
- **Modbus** - Legacy PLC polling (TCP/RTU)
- **DNP3** - Substation event-driven telemetry
- **BACnet** - Building automation
- **MQTT** - IoT sensor networks

## Safety
- WatchdogBuffer with hardware watchdog fallback
- Fallback policies: hold last state / safe-default / controlled shutdown

## Uplink
- Protobuf/gRPC over TLS
- SCADA protocols NEVER exposed to cloud

## Inference
Optional Jetson AGX support:
```rust
run_anomaly_detector();
extract_causal_features(); // ONNX
```

## Features
```toml
[dependencies]
edge = { path = "edge", features = ["tier2_inference"] }
```
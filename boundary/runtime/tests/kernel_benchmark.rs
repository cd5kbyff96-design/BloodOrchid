#[cfg(test)]
mod kernel_performance_tests {
    use std::time::Instant;
    use boundary_runtime::kernel::KernelBridge;
    use boundary_runtime::proto::SimulationState;

    fn sha256(bytes: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    #[test]
    fn kernel_determinism_1_step() {
        let first = KernelBridge::run_heat(1).unwrap();
        let second = KernelBridge::run_heat(1).unwrap();
        assert_eq!(first, second, "Kernel output must be deterministic for 1 step");
    }

    #[test]
    fn kernel_determinism_8_steps() {
        let first = KernelBridge::run_heat(8).unwrap();
        let second = KernelBridge::run_heat(8).unwrap();
        assert_eq!(first, second, "Kernel output must be deterministic for 8 steps");
    }

    #[test]
    fn kernel_determinism_100_steps() {
        let first = KernelBridge::run_heat(100).unwrap();
        let second = KernelBridge::run_heat(100).unwrap();
        assert_eq!(first, second, "Kernel output must be deterministic for 100 steps");
    }

    #[test]
    fn kernel_hash_stability_10_iterations() {
        let mut hashes = Vec::with_capacity(10);
        for _ in 0..10 {
            let result = KernelBridge::run_heat(5).unwrap();
            hashes.push(sha256(&result));
        }

        let first_hash = &hashes[0];
        for (i, hash) in hashes.iter().enumerate().skip(1) {
            assert_eq!(hash, first_hash, "Iteration {} hash differs from first", i + 1);
        }
    }

    #[test]
    fn kernel_performance_baseline_10_steps() {
        let start = Instant::now();
        let _result = KernelBridge::run_heat(10).unwrap();
        let elapsed = start.elapsed();

        println!("[BENCH] 10 steps took: {:?}", elapsed);

        assert!(elapsed.as_secs() < 5, "Kernel should complete 10 steps within 5 seconds");
    }

    #[test]
    fn kernel_performance_baseline_100_steps() {
        let start = Instant::now();
        let _result = KernelBridge::run_heat(100).unwrap();
        let elapsed = start.elapsed();

        println!("[BENCH] 100 steps took: {:?}", elapsed);

        assert!(elapsed.as_secs() < 30, "Kernel should complete 100 steps within 30 seconds");
    }

    #[test]
    fn kernel_output_produces_valid_state() {
        let bytes = KernelBridge::run_heat(5).unwrap();
        let state = SimulationState::decode(&bytes);

        assert!(state.is_ok(), "Kernel output must be valid protobuf");
        let state = state.unwrap();

        assert_eq!(state.step_index, 5);
        assert_eq!(state.simulation_id, "mves-heat-2d");
        assert_eq!(state.solver_kind, "heat_reference");
        assert!((state.simulation_time - 0.5).abs() < 0.001);

        assert!(state.primary_field.is_some());
        let field = state.primary_field.unwrap();
        assert_eq!(field.width, 12);
        assert_eq!(field.height, 12);
        assert_eq!(field.values.len(), 144);

        for val in &field.values {
            assert!(!val.is_nan(), "Field values must not be NaN");
            assert!(val.is_finite(), "Field values must be finite");
        }
    }

    #[test]
    fn kernel_step_increases_simulation_time() {
        for step in [1, 5, 10, 50] {
            let bytes = KernelBridge::run_heat(step).unwrap();
            let state = SimulationState::decode(&bytes).unwrap();
            let expected_time = step as f64 * 0.1;

            assert!((state.simulation_time - expected_time).abs() < 0.001,
                "Step {} should have time {}", step, expected_time);
        }
    }

    #[test]
    fn kernel_field_values_change_over_time() {
        let state1 = KernelBridge::run_heat(1).unwrap();
        let state1 = SimulationState::decode(&state1).unwrap();
        let values1 = &state1.primary_field.as_ref().unwrap().values;

        let state5 = KernelBridge::run_heat(5).unwrap();
        let state5 = SimulationState::decode(&state5).unwrap();
        let values5 = &state5.primary_field.as_ref().unwrap().values;

        assert_ne!(values1, values5, "Field values should change between steps");
    }

    #[test]
    fn kernel_boundary_conditions_preserved() {
        let bytes = KernelBridge::run_heat(10).unwrap();
        let state = SimulationState::decode(&bytes).unwrap();
        let field = state.primary_field.unwrap();
        let values = &field.values;

        for x in 0..12 {
            assert!((values[x] - 0.0).abs() < 0.001, "Top boundary should be ~0 at x={}", x);
            assert!((values[11 * 12 + x] - 0.0).abs() < 0.001,
                "Bottom boundary should be ~0 at x={}", x);
        }

        for y in 0..12 {
            assert!((values[y * 12] - 0.0).abs() < 0.001, "Left boundary should be ~0 at y={}", y);
            assert!((values[y * 12 + 11] - 0.0).abs() < 0.001,
                "Right boundary should be ~0 at y={}", y);
        }
    }

    #[test]
    fn kernel_memory_usage_reasonable() {
        let bytes = KernelBridge::run_heat(1000).unwrap();

        assert!(bytes.len() < 10 * 1024 * 1024,
            "Kernel output should be under 10MB even for 1000 steps");
    }
}
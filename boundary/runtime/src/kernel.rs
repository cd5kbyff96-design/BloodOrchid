use std::time::Duration;

pub mod ffi {
    use std::slice;

    unsafe extern "C" {
        fn mves_kernel_run_heat(
            steps: u64,
            out_ptr: *mut *const u8,
            out_len: *mut usize,
        ) -> bool;
        fn mves_kernel_free_buffer(ptr: *const u8, len: usize);
        fn mves_kernel_advance_state(
            input_state: *const f32,
            input_size: usize,
            steps: u64,
            output_state: *mut *mut f32,
            output_size: *mut usize,
        ) -> bool;
        fn mves_kernel_free_state(state_ptr: *mut f32);
    }

    pub fn run_heat(steps: u64) -> Result<Vec<u8>, String> {
        let mut ptr = std::ptr::null();
        let mut len = 0usize;
        let success = unsafe { mves_kernel_run_heat(steps, &mut ptr, &mut len) };
        if !success {
            return Err("kernel execution failed".into());
        }
        if ptr.is_null() || len == 0 {
            return Err("kernel returned empty frame".into());
        }
        let bytes = unsafe { slice::from_raw_parts(ptr, len).to_vec() };
        unsafe { mves_kernel_free_buffer(ptr, len) };
        Ok(bytes)
    }
}

pub struct KernelBridge;

impl KernelBridge {
    pub fn run_heat(steps: u64) -> Result<Vec<u8>, String> {
        run_heat_with_timeout(steps, Duration::from_secs(5))
    }
}

fn run_heat_with_timeout(steps: u64, timeout: Duration) -> Result<Vec<u8>, String> {
    use std::thread;
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    let _ = thread::spawn(move || {
        let result = ffi::run_heat(steps);
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(_) => Err(format!("kernel execution timed out after {:?}", timeout)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_bridge_returns_bytes() {
        let result = KernelBridge::run_heat(1);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn kernel_output_is_deterministic() {
        let first = KernelBridge::run_heat(8).unwrap();
        let second = KernelBridge::run_heat(8).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn kernel_output_parses_as_simulation_state() {
        use crate::proto::SimulationState;
        let bytes = KernelBridge::run_heat(4).unwrap();
        let state = SimulationState::decode(&bytes);
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(state.step_index, 4);
        assert_eq!(state.simulation_id, "mves-heat-2d");
    }
}
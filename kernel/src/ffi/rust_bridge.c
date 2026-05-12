/**
 * kernel/src/ffi/rust_bridge.c
 * Rust FFI Bridge - Public interface for Rust boundary
 */

#include "veiliris/kernel/ffi/kernel_c_api.h"
#include "veiliris/kernel/core/runtime/execution_engine.h"
#include "veiliris/kernel/core/bootstrap.h"
#include <stdlib.h>
#include <string.h>

static KernelRuntime g_runtime = {0};
static KernelConfig g_config = {0};

const KernelConfig kDefaultConfig = {
    .width = 12,
    .height = 12,
    .spacing = 1.0f,
    .dt = 0.1,
    .diffusion = 0.15,
    .seed = 0xDEADBEEFCAFEBABE
};

int veiliris_kernel_init(const KernelConfig* config) {
    if (g_runtime.initialized) {
        veiliris_kernel_shutdown();
    }
    
    if (!config) {
        config = &kDefaultConfig;
    }
    
    g_config = *config;
    return execution_engine_init(&g_runtime, &g_config);
}

int veiliris_kernel_run(uint64_t steps) {
    return execution_engine_step(&g_runtime, &g_config, steps);
}

int veiliris_kernel_get_state(const float** state, size_t* size) {
    if (!g_runtime.initialized) return -1;
    return execution_engine_get_state(&g_runtime, (void**)state, size);
}

int veiliris_kernel_set_state(const float* state, size_t size) {
    if (!g_runtime.initialized) return -1;
    return execution_engine_set_state(&g_runtime, state, size);
}

uint64_t veiliris_kernel_get_step(void) {
    return g_runtime.current_step;
}

int veiliris_kernel_shutdown(void) {
    return kernel_shutdown(&g_runtime);
}
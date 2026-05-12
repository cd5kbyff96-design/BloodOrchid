/**
 * kernel/src/core/runtime/execution_engine.c
 */

#include "veiliris/kernel/core/runtime/execution_engine.h"
#include "veiliris/kernel/core/bootstrap.h"
#include <math.h>
#include <string.h>

static int run_heat_step(float* field, size_t width, size_t height, double alpha) {
    float* next = (float*)malloc(width * height * sizeof(float));
    if (!next) return -1;
    
    for (size_t y = 1; y + 1 < height; ++y) {
        for (size_t x = 1; x + 1 < width; ++x) {
            size_t idx = y * width + x;
            float center = field[idx];
            float left = field[idx - 1];
            float right = field[idx + 1];
            float up = field[idx - width];
            float down = field[idx + width];
            
            float laplacian = left + right + up + down - 4.0f * center;
            next[idx] = center + (float)(alpha * laplacian);
        }
    }
    
    memcpy(field, next, width * height * sizeof(float));
    free(next);
    return 0;
}

int execution_engine_init(KernelRuntime* runtime, const KernelConfig* config) {
    return kernel_bootstrap(config, runtime);
}

int execution_engine_step(KernelRuntime* runtime, const KernelConfig* config, uint64_t steps) {
    if (!runtime || !runtime->initialized || !config) return -1;
    
    float* field = (float*)runtime->state_buffer;
    double alpha = config->diffusion * config->dt;
    
    for (uint64_t i = 0; i < steps; ++i) {
        if (run_heat_step(field, config->width, config->height, alpha) != 0) {
            return -2;
        }
        runtime->current_step++;
    }
    
    return 0;
}

int execution_engine_get_state(const KernelRuntime* runtime, void** out_state, size_t* out_size) {
    if (!runtime || !runtime->initialized || !out_state || !out_size) return -1;
    
    *out_state = runtime->state_buffer;
    *out_size = runtime->buffer_size;
    return 0;
}

int execution_engine_set_state(KernelRuntime* runtime, const void* state, size_t size) {
    if (!runtime || !runtime->initialized || !state || size != runtime->buffer_size) return -1;
    
    memcpy(runtime->state_buffer, state, size);
    return 0;
}
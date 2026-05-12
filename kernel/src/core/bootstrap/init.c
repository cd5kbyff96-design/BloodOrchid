/**
 * kernel/src/core/bootstrap/init.c
 * Blood Orchid Kernel Bootstrap Implementation
 */

#include "veiliris/kernel/core/bootstrap.h"
#include "veiliris/kernel/core/memory/memory_manager.h"
#include <stdlib.h>
#include <string.h>

const KernelConfig kDefaultKernelConfig = {
    .width = 12,
    .height = 12,
    .spacing = 1.0f,
    .dt = 0.1,
    .diffusion = 0.15,
    .seed = 0xDEADBEEFCAFEBABE
};

int kernel_bootstrap(const KernelConfig* config, KernelRuntime* runtime) {
    if (config == NULL || runtime == NULL) {
        return -1;
    }
    
    memset(runtime, 0, sizeof(KernelRuntime));
    
    size_t field_size = config->width * config->height * sizeof(float);
    runtime->state_buffer = aligned_alloc(32, field_size);
    if (runtime->state_buffer == NULL) {
        return -2;
    }
    
    runtime->buffer_size = field_size;
    runtime->current_step = 0;
    runtime->initialized = 1;
    
    /* Initialize field with Gaussian pulse */
    float* values = (float*)runtime->state_buffer;
    size_t center_x = config->width / 2;
    size_t center_y = config->height / 2;
    
    for (size_t y = 0; y < config->height; ++y) {
        for (size_t x = 0; x < config->width; ++x) {
            size_t idx = y * config->width + x;
            double dx = (double)x - (double)center_x;
            double dy = (double)y - (double)center_y;
            values[idx] = (float)exp(-0.20 * (dx * dx + dy * dy));
        }
    }
    
    return 0;
}

int kernel_shutdown(KernelRuntime* runtime) {
    if (runtime == NULL || !runtime->initialized) {
        return -1;
    }
    
    free(runtime->state_buffer);
    runtime->state_buffer = NULL;
    runtime->buffer_size = 0;
    runtime->current_step = 0;
    runtime->initialized = 0;
    
    return 0;
}

int kernel_is_initialized(const KernelRuntime* runtime) {
    return (runtime != NULL && runtime->initialized);
}
/**
 * kernel/include/veiliris/kernel/ffi/kernel_c_api.h
 */

#ifndef VEILIRIS_KERNEL_FFI_KERNEL_C_API_H
#define VEILIRIS_KERNEL_FFI_KERNEL_C_API_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque kernel runtime pointer */
typedef struct KernelRuntime KernelRuntime;

/* Configuration structure */
typedef struct {
    size_t width;
    size_t height;
    float spacing;
    double dt;
    double diffusion;
    uint64_t seed;
} KernelConfig;

/* 
 * Initialize kernel with configuration
 * Returns 0 on success, non-zero on failure
 */
int veiliris_kernel_init(const KernelConfig* config);

/*
 * Run simulation for specified steps
 * Returns 0 on success, non-zero on failure
 */
int veiliris_kernel_run(uint64_t steps);

/*
 * Get current state buffer
 * Caller must NOT free the returned pointer
 */
int veiliris_kernel_get_state(const float** state, size_t* size);

/*
 * Set state buffer (replaces current state)
 */
int veiliris_kernel_set_state(const float* state, size_t size);

/*
 * Get current step count
 */
uint64_t veiliris_kernel_get_step(void);

/*
 * Shutdown kernel and free resources
 */
int veiliris_kernel_shutdown(void);

/* Default configuration */
extern const KernelConfig kDefaultConfig;

#ifdef __cplusplus
}
#endif

#endif
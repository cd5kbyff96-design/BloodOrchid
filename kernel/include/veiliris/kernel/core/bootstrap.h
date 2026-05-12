/**
 * kernel/include/veiliris/kernel/core/bootstrap.h
 * Blood Orchid Kernel Core Bootstrap
 */

#ifndef VEILIRIS_KERNEL_CORE_BOOTSTRAP_H
#define VEILIRIS_KERNEL_CORE_BOOTSTRAP_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Bootstrap configuration */
typedef struct {
    size_t width;
    size_t height;
    float spacing;
    double dt;
    double diffusion;
    uint64_t seed;
} KernelConfig;

/* Runtime state */
typedef struct {
    void* state_buffer;
    size_t buffer_size;
    size_t current_step;
    int initialized;
} KernelRuntime;

/* Bootstrap functions */
int kernel_bootstrap(const KernelConfig* config, KernelRuntime* runtime);
int kernel_shutdown(KernelRuntime* runtime);
int kernel_is_initialized(const KernelRuntime* runtime);

/* Default configuration */
extern const KernelConfig kDefaultKernelConfig;

#ifdef __cplusplus
}
#endif

#endif /* VEILIRIS_KERNEL_CORE_BOOTSTRAP_H */
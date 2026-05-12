/**
 * kernel/include/veiliris/kernel/core/runtime/execution_engine.h
 */

#ifndef VEILIRIS_KERNEL_CORE_RUNTIME_EXECUTION_ENGINE_H
#define VEILIRIS_KERNEL_CORE_RUNTIME_EXECUTION_ENGINE_H

#include "veiliris/kernel/core/bootstrap.h"
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    SOLVER_HEAT_2D = 1,
    SOLVER_NAVIER_STOKES = 2,
    SOLVER_EULER_MARUYAMA = 3
} SolverType;

typedef struct {
    SolverType type;
    double current_time;
    uint64_t step_count;
} ExecutionContext;

int execution_engine_init(KernelRuntime* runtime, const KernelConfig* config);
int execution_engine_step(KernelRuntime* runtime, const KernelConfig* config, uint64_t steps);
int execution_engine_get_state(const KernelRuntime* runtime, void** out_state, size_t* out_size);
int execution_engine_set_state(KernelRuntime* runtime, const void* state, size_t size);

#ifdef __cplusplus
}
#endif

#endif
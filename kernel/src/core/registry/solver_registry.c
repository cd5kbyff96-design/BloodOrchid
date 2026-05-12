/**
 * kernel/src/core/registry/solver_registry.c
 */

#include <string.h>
#include <stdlib.h>

typedef enum {
    SOLVER_UNKNOWN = 0,
    SOLVER_HEAT_2D,
    SOLVER_NAVIER_STOKES,
    SOLVER_EULER_MARUYAMA
} SolverId;

typedef int (*SolverFunc)(void* state, size_t width, size_t height, 
                          double dt, double diffusion, uint64_t steps);

typedef struct {
    const char* name;
    SolverId id;
    SolverFunc func;
} SolverEntry;

static SolverEntry solver_registry[] = {
    {"heat_2d", SOLVER_HEAT_2D, NULL},
    {"navier_stokes", SOLVER_NAVIER_STOKES, NULL},
    {"euler_maruyama", SOLVER_EULER_MARUYAMA, NULL},
    {NULL, SOLVER_UNKNOWN, NULL}
};

SolverId solver_lookup(const char* name) {
    for (int i = 0; solver_registry[i].name; ++i) {
        if (strcmp(solver_registry[i].name, name) == 0) {
            return solver_registry[i].id;
        }
    }
    return SOLVER_UNKNOWN;
}

const char* solver_name(SolverId id) {
    for (int i = 0; solver_registry[i].name; ++i) {
        if (solver_registry[i].id == id) {
            return solver_registry[i].name;
        }
    }
    return "unknown";
}

int solver_register(SolverId id, SolverFunc func) {
    for (int i = 0; solver_registry[i].name; ++i) {
        if (solver_registry[i].id == id) {
            solver_registry[i].func = func;
            return 0;
        }
    }
    return -1;
}

int solver_execute(SolverId id, void* state, size_t width, size_t height,
                   double dt, double diffusion, uint64_t steps) {
    for (int i = 0; solver_registry[i].name; ++i) {
        if (solver_registry[i].id == id && solver_registry[i].func) {
            return solver_registry[i].func(state, width, height, 
                                           dt, diffusion, steps);
        }
    }
    return -1;
}
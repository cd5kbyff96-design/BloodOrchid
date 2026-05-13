/**
 * kernel/include/veiliris/kernel/core/solver/heat_2d_solver.h
 * Vail Iris Blood Orchid - Heat Equation 2D Solver Interface
 * Comprehensive solver interface with multiple boundary conditions and methods
 */

#ifndef VEILIRIS_HEAT_2D_SOLVER_H
#define VEILIRIS_HEAT_2D_SOLVER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Boundary condition types
typedef enum {
    HEAT_BC_DIRICHLET = 0,
    HEAT_BC_NEUMANN = 1,
    HEAT_BC_PERIODIC = 2,
    HEAT_BC_MIXED = 3,
    HEAT_BC_ROBIN = 4
} heat_bc_type_t;

// Solver method types
typedef enum {
    HEAT_METHOD_EXPLICIT = 0,
    HEAT_METHOD_IMPLICIT = 1,
    HEAT_METHOD_CRANK_NICOLSON = 2,
    HEAT_METHOD_ADAPTIVE = 3
} heat_method_t;

// Solver configuration
typedef struct {
    size_t width;
    size_t height;
    double dx;
    double dy;
    double dt;
    double alpha;
    double max_time;
    double tolerance;
    uint64_t max_iterations;
    heat_bc_type_t bc_type;
    heat_method_t method;
    int use_output;
    int use_parallel;
    int use_adaptive;
    char output_path[256];
    void (*source_callback)(double x, double y, double t, double *value);
    void (*boundary_callback)(double x, double y, double *value);
} heat_solver_config_t;

// Solver state
typedef struct heat_solver_state heat_solver_state_t;

// Initialize solver with configuration
heat_solver_state_t* heat_solver_init(const heat_solver_config_t *config);

// Initialize solver with default parameters
heat_solver_state_t* heat_solver_init_default(size_t width, size_t height);

// Free solver state
void heat_solver_free(heat_solver_state_t *state);

// Run heat equation solver
int heat_solver_run(heat_solver_state_t *state);

// Step solver by one timestep
int heat_solver_step(heat_solver_state_t *state);

// Run explicit method
int heat_solver_explicit(heat_solver_state_t *state, uint64_t steps);

// Run implicit method
int heat_solver_implicit(heat_solver_state_t *state, uint64_t steps);

// Run Crank-Nicolson method
int heat_solver_crank_nicolson(heat_solver_state_t *state, uint64_t steps);

// Set source term
int heat_solver_set_source(heat_solver_state_t *state,
                            size_t x, size_t y, double strength);

// Set boundary condition
int heat_solver_set_boundary(heat_solver_state_t *state,
                              size_t x, size_t y, double value);

// Get field values
const float* heat_solver_get_field(const heat_solver_state_t *state);

// Get field values copy
int heat_solver_copy_field(const heat_solver_state_t *state, 
                            float *output, size_t max_size);

// Set field values
int heat_solver_set_field(heat_solver_state_t *state,
                            const float *field, size_t size);

// Get solver statistics
int heat_solver_get_stats(const heat_solver_state_t *state,
                           double *min_val, double *max_val, double *mean_val);

// Output field to file
int heat_solver_output(heat_solver_state_t *state, const char *filename);

// Set output callback
typedef void (*heat_output_callback_t)(const float *field, 
                                          size_t width, size_t height, 
                                          uint64_t step, double time);

int heat_solver_set_output_callback(heat_solver_state_t *state,
                                     heat_output_callback_t callback);

// Adaptive timestep calculation
double heat_solver_compute_cfl_timestep(double dx, double dy, double alpha);

// Error estimation
double heat_solver_compute_error(const float *field1, const float *field2,
                                  size_t width, size_t height);

// Convergence check
int heat_solver_check_convergence(const heat_solver_state_t *state, double tolerance);

// Get current iteration
uint64_t heat_solver_get_iteration(const heat_solver_state_t *state);

// Get current simulation time
double heat_solver_get_time(const heat_solver_state_t *state);

// Reset solver state
int heat_solver_reset(heat_solver_state_t *state);

// Initialize Gaussian distribution
int heat_solver_initialize_gaussian(heat_solver_state_t *state,
                                     double center_x, double center_y, double sigma);

// Initialize random field
int heat_solver_initialize_random(heat_solver_state_t *state, double amplitude);

// Initialize with file
int heat_solver_initialize_file(heat_solver_state_t *state, const char *filename);

#ifdef __cplusplus
}
#endif

#endif /* VEILIRIS_HEAT_2D_SOLVER_H */
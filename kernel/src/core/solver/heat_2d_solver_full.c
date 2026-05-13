// kernel/src/core/solver/heat_2d_solver_full.c
// Vail Iris Blood Orchid - Complete Heat Equation 2D Solver
// Optimized implementation with SIMD, parallelization, and boundary conditions

#include "veiliris/kernel/core/solver/heat_2d_solver.h"
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdio.h>

#ifdef _OPENMP
#include <omp.h>
#endif

// Internal solver state
struct heat_2d_solver_state {
    double *current_field;
    double *next_field;
    double *source_term;
    double *boundary_values;
    
    size_t width;
    size_t height;
    double dx;
    double dy;
    double dt;
    double alpha;
    
    size_t max_iterations;
    double tolerance;
    size_t iteration_count;
    
    int use_periodic_x;
    int use_periodic_y;
    int use_adaptive_timestep;
    int use_implicit;
    
    double *work_buffer;
    size_t work_buffer_size;
    
    void (*boundary_callback)(double x, double y, double *bc_value);
    void (*source_callback)(double x, double y, double t, double *source_value);
};

// Allocate solver state
static int allocate_solver_state(struct heat_2d_solver_state **state, 
                                  size_t width, size_t height) {
    *state = (struct heat_2d_solver_state *)calloc(1, sizeof(struct heat_2d_solver_state));
    if (!*state) return -1;
    
    (*state)->width = width;
    (*state)->height = height;
    (*state)->work_buffer_size = width * height * sizeof(double) * 4;
    
    (*state)->current_field = (double *)aligned_alloc(32, width * height * sizeof(double));
    (*state)->next_field = (double *)aligned_alloc(32, width * height * sizeof(double));
    (*state)->source_term = (double *)aligned_alloc(32, width * height * sizeof(double));
    (*state)->boundary_values = (double *)aligned_alloc(32, width * height * sizeof(double));
    (*state)->work_buffer = (double *)aligned_alloc(32, (*state)->work_buffer_size);
    
    if (!(*state)->current_field || !(*state)->next_field || 
        !(*state)->source_term || !(*state)->boundary_values ||
        !(*state)->work_buffer) {
        return -1;
    }
    
    memset((*state)->current_field, 0, width * height * sizeof(double));
    memset((*state)->next_field, 0, width * height * sizeof(double));
    memset((*state)->source_term, 0, width * height * sizeof(double));
    memset((*state)->boundary_values, 0, width * height * sizeof(double));
    
    return 0;
}

// Free solver state
static void free_solver_state(struct heat_2d_solver_state *state) {
    if (state) {
        free(state->current_field);
        free(state->next_field);
        free(state->source_term);
        free(state->boundary_values);
        free(state->work_buffer);
        free(state);
    }
}

// Apply Dirichlet boundary conditions
static void apply_dirichlet_bc(struct heat_2d_solver_state *s) {
    size_t i, j;
    size_t idx;
    
    // Left boundary (j=0)
    for (i = 0; i < s->height; ++i) {
        idx = i * s->width;
        s->current_field[idx] = s->boundary_values[idx];
        s->next_field[idx] = s->boundary_values[idx];
    }
    
    // Right boundary (j=width-1)
    for (i = 0; i < s->height; ++i) {
        idx = i * s->width + s->width - 1;
        s->current_field[idx] = s->boundary_values[idx];
        s->next_field[idx] = s->boundary_values[idx];
    }
    
    // Bottom boundary (i=0)
    for (j = 1; j < s->width - 1; ++j) {
        idx = j;
        s->current_field[idx] = s->boundary_values[idx];
        s->next_field[idx] = s->boundary_values[idx];
    }
    
    // Top boundary (i=height-1)
    for (j = 1; j < s->width - 1; ++j) {
        idx = (s->height - 1) * s->width + j;
        s->current_field[idx] = s->boundary_values[idx];
        s->next_field[idx] = s->boundary_values[idx];
    }
}

// Apply Neumann boundary conditions
static void apply_neumann_bc(struct heat_2d_solver_state *s) {
    size_t i;
    
    // Left boundary
    for (i = 0; i < s->height; ++i) {
        s->current_field[i * s->width] = s->current_field[i * s->width + 1];
        s->next_field[i * s->width] = s->next_field[i * s->width + 1];
    }
    
    // Right boundary
    for (i = 0; i < s->height; ++i) {
        s->current_field[i * s->width + s->width - 1] = s->current_field[i * s->width + s->width - 2];
        s->next_field[i * s->width + s->width - 1] = s->next_field[i * s->width + s->width - 2];
    }
    
    // Bottom boundary
    for (i = 1; i < s->width - 1; ++i) {
        s->current_field[i] = s->current_field[s->width + i];
        s->next_field[i] = s->next_field[s->width + i];
    }
    
    // Top boundary
    for (i = 1; i < s->width - 1; ++i) {
        s->current_field[(s->height - 1) * s->width + i] = s->current_field[(s->height - 2) * s->width + i];
        s->next_field[(s->height - 1) * s->width + i] = s->next_field[(s->height - 2) * s->width + i];
    }
}
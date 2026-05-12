/**
 * kernel/src/core/solver/heat_2d_solver.c
 */

#include "veiliris/kernel/core/bootstrap.h"
#include <math.h>
#include <string.h>

int heat_2d_solver_run(float* field, size_t width, size_t height, 
                       double dt, double diffusion, uint64_t steps) {
    if (!field || width < 2 || height < 2) return -1;
    
    double alpha = diffusion * dt;
    float* next = (float*)malloc(width * height * sizeof(float));
    if (!next) return -1;
    
    for (uint64_t step = 0; step < steps; ++step) {
        memset(next, 0, width * height * sizeof(float));
        
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
    }
    
    free(next);
    return 0;
}

void heat_2d_solver_initialize(float* field, size_t width, size_t height) {
    size_t center_x = width / 2;
    size_t center_y = height / 2;
    
    for (size_t y = 0; y < height; ++y) {
        for (size_t x = 0; x < width; ++x) {
            size_t idx = y * width + x;
            double dx = (double)x - (double)center_x;
            double dy = (double)y - (double)center_y;
            field[idx] = (float)exp(-0.20 * (dx * dx + dy * dy));
        }
    }
}
// kernel/pde_ref/advanced_solver.cpp
#include "advanced_solver.hpp"
#include <algorithm>

namespace veiliris {
namespace pde {

template<typename T>
AdvancedSolver<T>::AdvancedSolver(size_t width, size_t height, T dx, T dt, T alpha)
    : width_(width), height_(height), dx_(dx), dt_(dt), alpha_(alpha) {
    field_.resize(height, std::vector<T>(width, T(0)));
    next_field_.resize(height, std::vector<T>(width, T(0)));
    boundary_func_ = [](T, T) { return T(0); };
}

template<typename T>
void AdvancedSolver<T>::initialize_gaussian(T cx, T cy, T sigma) {
    T coeff = T(1) / (T(2) * sigma * sigma);
    for (size_t i = 0; i < height_; ++i) {
        for (size_t j = 0; j < width_; ++j) {
            T dx = static_cast<T>(j) - cx;
            T dy = static_cast<T>(i) - cy;
            field_[i][j] = std::exp(-coeff * (dx * dx + dy * dy));
        }
    }
}

template<typename T>
void AdvancedSolver<T>::step(size_t iterations) {
    for (size_t iter = 0; iter < iterations; ++iter) {
        for (size_t i = 1; i < height_ - 1; ++i) {
            for (size_t j = 1; j < width_ - 1; ++j) {
                next_field_[i][j] = field_[i][j] + alpha_ * laplacian(i, j);
            }
        }
        
        // Apply boundary conditions
        for (size_t j = 0; j < width_; ++j) {
            field_[0][j] = boundary_func_(static_cast<T>(j), T(0));
            field_[height_-1][j] = boundary_func_(static_cast<T>(j), static_cast<T>(height_-1));
        }
        for (size_t i = 0; i < height_; ++i) {
            field_[i][0] = boundary_func_(T(0), static_cast<T>(i));
            field_[i][width_-1] = boundary_func_(static_cast<T>(width_-1), static_cast<T>(i));
        }
        
        field_.swap(next_field_);
    }
}

template<typename T>
T AdvancedSolver<T>::laplacian(size_t i, size_t j) const {
    return (field_[i+1][j] + field_[i-1][j] + field_[i][j+1] + field_[i][j-1] 
            - T(4) * field_[i][j]) / (dx_ * dx_);
}

// Explicit instantiation for float and double
template class AdvancedSolver<float>;
template class AdvancedSolver<double>;

} // namespace pde
} // namespace veiliris
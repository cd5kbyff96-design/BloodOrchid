// kernel/pde_ref/implicit_solver.hpp
#ifndef VEILIRIS_IMPLICIT_SOLVER_HPP
#define VEILIRIS_IMPLICIT_SOLVER_HPP

#include <vector>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
class ImplicitSolver {
public:
    // Solve (I - alpha * dt * Laplacian) u_new = u_old
    static std::vector<std::vector<T>> solve_heat_implicit(
        const std::vector<std::vector<T>>& u_old,
        T dx,
        T dt,
        T alpha,
        size_t iterations = 100,
        T tolerance = T(1e-10)
    ) {
        size_t n = u_old.size();
        size_t m = u_old[0].size();
        std::vector<std::vector<T>> u_new = u_old;
        std::vector<std::vector<T>> u_prev = u_old;
        
        T ratio = alpha * dt / (dx * dx);
        
        for (size_t iter = 0; iter < iterations; ++iter) {
            for (size_t i = 1; i < n - 1; ++i) {
                for (size_t j = 1; j < m - 1; ++j) {
                    u_new[i][j] = u_old[i][j] + ratio * 
                        (u_prev[i+1][j] + u_new[i+1][j] +
                         u_prev[i-1][j] + u_new[i-1][j] +
                         u_prev[i][j+1] + u_new[i][j+1] +
                         u_prev[i][j-1] + u_new[i][j-1] - 
                         4.0 * u_new[i][j]) / T(4.0);
                }
            }
            
            // Check convergence
            T error = T(0);
            for (size_t i = 0; i < n; ++i) {
                for (size_t j = 0; j < m; ++j) {
                    error = std::max(error, std::abs(u_new[i][j] - u_prev[i][j]));
                }
            }
            
            u_prev = u_new;
            if (error < tolerance) break;
        }
        
        return u_new;
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_IMPLICIT_SOLVER_HPP
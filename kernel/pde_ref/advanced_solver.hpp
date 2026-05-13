// kernel/pde_ref/advanced_solver.hpp
#ifndef VEILIRIS_ADVANCED_SOLVER_HPP
#define VEILIRIS_ADVANCED_SOLVER_HPP

#include <vector>
#include <array>
#include <memory>
#include <functional>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
class AdvancedSolver {
public:
    using Grid = std::vector<std::vector<T>>;
    
    AdvancedSolver(size_t width, size_t height, T dx, T dt, T alpha);
    
    void initialize_gaussian(T center_x, T center_y, T sigma);
    void step(size_t iterations);
    
    const Grid& get_field() const { return field_; }
    void set_boundary_condition(std::function<T(T, T)> bc_func);
    
    T laplacian(size_t i, size_t j) const;
    
private:
    Grid field_;
    Grid next_field_;
    size_t width_;
    size_t height_;
    T dx_;
    T dt_;
    T alpha_;
    std::function<T(T, T)> boundary_func_;
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_ADVANCED_SOLVER_HPP
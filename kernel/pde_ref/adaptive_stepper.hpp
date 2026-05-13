// kernel/pde_ref/adaptive_stepper.hpp
#ifndef VEILIRIS_ADAPTIVE_STEPPER_HPP
#define VEILIRIS_ADAPTIVE_STEPPER_HPP

#include <vector>
#include <cmath>
#include <limits>

namespace veiliris {
namespace pde {

template<typename T>
class AdaptiveStepper {
public:
    struct StepResult {
        T new_dt;
        T error_estimate;
        bool accepted;
        size_t iterations;
    };
    
    AdaptiveStepper(T initial_dt, T min_dt, T max_dt, T tolerance);
    
    template<typename Solver>
    StepResult step_with_adaptation(Solver& solver, size_t substeps = 10) {
        auto initial_state = solver.get_field();
        solver.step(substeps);
        
        T max_change = T(0);
        const auto& final_state = solver.get_field();
        
        for (size_t i = 0; i < initial_state.size(); ++i) {
            for (size_t j = 0; j < initial_state[i].size(); ++j) {
                T change = std::abs(final_state[i][j] - initial_state[i][j]);
                max_change = std::max(max_change, change);
            }
        }
        
        T error = max_change * static_cast<T>(substeps);
        bool accepted = error < tolerance_;
        T new_dt = accepted ? std::min(dt_ * T(1.1), max_dt_) : dt_ * T(0.5);
        
        return {new_dt, error, accepted, substeps};
    }
    
    void reset_dt() { dt_ = initial_dt_; }
    T get_dt() const { return dt_; }
    
private:
    T dt_;
    T initial_dt_;
    T min_dt_;
    T max_dt_;
    T tolerance_;
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_ADAPTIVE_STEPPER_HPP
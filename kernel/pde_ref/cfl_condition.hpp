// kernel/pde_ref/cfl_condition.hpp
#ifndef VEILIRIS_CFL_CONDITION_HPP
#define VEILIRIS_CFL_CONDITION_HPP

#include <cmath>
#include <limits>

namespace veiliris {
namespace pde {

template<typename T>
class CFLCondition {
public:
    struct CFLData {
        T dt_max;
        T dt_current;
        T cfl_number;
        bool stable;
    };
    
    static CFLData check_heat_equation(T dx, T dt, T alpha) {
        T dt_max = dx * dx / (T(4) * alpha);  // 2D heat equation
        T cfl = dt / dt_max;
        return {dt_max, dt, cfl, cfl <= T(1.0)};
    }
    
    static CFLData check_advection(T dx, T dt, T velocity) {
        T dt_max = dx / std::abs(velocity);
        T cfl = dt / dt_max;
        return {dt_max, dt, cfl, cfl <= T(1.0)};
    }
    
    static CFLData check_wave(T dx, T dt, T c) {
        T dt_max = dx / (c * std::sqrt(T(2)));  // 2D wave equation
        T cfl = dt / dt_max;
        return {dt_max, dt, cfl, cfl <= T(1.0)};
    }
    
    static T compute_stable_dt(T dx, T alpha, T safety_factor = T(0.9)) {
        return safety_factor * dx * dx / (T(4) * (alpha + std::numeric_limits<T>::epsilon()));
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_CFL_CONDITION_HPP
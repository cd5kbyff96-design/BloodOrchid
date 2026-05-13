// kernel/pde_ref/error_estimator.hpp
#ifndef VEILIRIS_ERROR_ESTIMATOR_HPP
#define VEILIRIS_ERROR_ESTIMATOR_HPP

#include <vector>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
class ErrorEstimator {
public:
    struct ErrorMetrics {
        T l2_error;
        T linf_error;
        T convergence_rate;
        size_t grid_points;
    };
    
    static ErrorMetrics estimate(
        const std::vector<std::vector<T>>& computed,
        const std::vector<std::vector<T>>& reference
    ) {
        size_t n = computed.size();
        size_t m = computed[0].size();
        
        T l2_sum = T(0);
        T linf_max = T(0);
        
        for (size_t i = 0; i < n; ++i) {
            for (size_t j = 0; j < m; ++j) {
                T diff = computed[i][j] - reference[i][j];
                l2_sum += diff * diff;
                linf_max = std::max(linf_max, std::abs(diff));
            }
        }
        
        return {
            std::sqrt(l2_sum / (n * m)),
            linf_max,
            T(0),  // Computed separately
            n * m
        };
    }
    
    static T richardson_extrapolation(T coarse, T fine, T refinement_ratio = T(2)) {
        return (refinement_ratio * refinement_ratio * coarse - fine) / 
               (refinement_ratio * refinement_ratio - T(1));
    }
    
    static T convergence_rate(T error1, T error2, T dt1, T dt2) {
        if (error1 == T(0) || error2 == T(0) || dt1 == dt(2)) return T(0);
        return std::log(error2 / error1) / std::log(dt2 / dt1);
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_ERROR_ESTIMATOR_HPP
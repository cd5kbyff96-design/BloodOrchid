// kernel/pde_ref/fdm_operator.hpp
#ifndef VEILIRIS_FDM_OPERATOR_HPP
#define VEILIRIS_FDM_OPERATOR_HPP

#include <vector>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
class FDMOperator {
public:
    static T gradient_x(const std::vector<std::vector<T>>& field, size_t i, size_t j, T dx) {
        if (j == 0) return (field[i][j + 1] - field[i][j]) / dx;
        if (j == field[i].size() - 1) return (field[i][j] - field[i][j - 1]) / dx;
        return (field[i][j + 1] - field[i][j - 1]) / (T(2) * dx);
    }
    
    static T gradient_y(const std::vector<std::vector<T>>& field, size_t i, size_t j, T dy) {
        if (i == 0) return (field[i + 1][j] - field[i][j]) / dy;
        if (i == field.size() - 1) return (field[i][j] - field[i - 1][j]) / dy;
        return (field[i + 1][j] - field[i - 1][j]) / (T(2) * dy);
    }
    
    static T laplacian(const std::vector<std::vector<T>>& field, size_t i, size_t j, T dx, T dy) {
        T d2x = T(0), d2y = T(0);
        
        if (j > 0 && j < field[i].size() - 1) {
            d2x = (field[i][j + 1] - T(2) * field[i][j] + field[i][j - 1]) / (dx * dx);
        }
        if (i > 0 && i < field.size() - 1) {
            d2y = (field[i + 1][j] - T(2) * field[i][j] + field[i - 1][j]) / (dy * dy);
        }
        
        return d2x + d2y;
    }
    
    static T divergence(const std::vector<std::vector<T>>& u, const std::vector<std::vector<T>>& v,
                        size_t i, size_t j, T dx, T dy) {
        T du_dx = gradient_x(u, i, j, dx);
        T dv_dy = gradient_y(v, i, j, dy);
        return du_dx + dv_dy;
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_FDM_OPERATOR_HPP
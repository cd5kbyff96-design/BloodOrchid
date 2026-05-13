// kernel/pde_ref/field_operations.hpp
#ifndef VEILIRIS_FIELD_OPERATIONS_HPP
#define VEILIRIS_FIELD_OPERATIONS_HPP

#include <vector>
#include <cmath>
#include <algorithm>

namespace veiliris {
namespace pde {

template<typename T>
class FieldOperations {
public:
    static std::vector<std::vector<T>> add(
        const std::vector<std::vector<T>>& a,
        const std::vector<std::vector<T>>& b
    ) {
        size_t rows = a.size();
        size_t cols = a[0].size();
        std::vector<std::vector<T>> result(rows, std::vector<T>(cols));
        
        for (size_t i = 0; i < rows; ++i) {
            for (size_t j = 0; j < cols; ++j) {
                result[i][j] = a[i][j] + b[i][j];
            }
        }
        return result;
    }
    
    static std::vector<std::vector<T>> scale(
        const std::vector<std::vector<T>>& field,
        T factor
    ) {
        size_t rows = field.size();
        size_t cols = field[0].size();
        std::vector<std::vector<T>> result(rows, std::vector<T>(cols));
        
        for (size_t i = 0; i < rows; ++i) {
            for (size_t j = 0; j < cols; ++j) {
                result[i][j] = field[i][j] * factor;
            }
        }
        return result;
    }
    
    static T l2_norm(const std::vector<std::vector<T>>& field) {
        T sum = T(0);
        for (const auto& row : field) {
            for (T val : row) {
                sum += val * val;
            }
        }
        return std::sqrt(sum);
    }
    
    static T linf_norm(const std::vector<std::vector<T>>& field) {
        T max_val = T(0);
        for (const auto& row : field) {
            for (T val : row) {
                max_val = std::max(max_val, std::abs(val));
            }
        }
        return max_val;
    }
    
    static T dot_product(
        const std::vector<std::vector<T>>& a,
        const std::vector<std::vector<T>>& b
    ) {
        T sum = T(0);
        for (size_t i = 0; i < a.size(); ++i) {
            for (size_t j = 0; j < a[i].size(); ++j) {
                sum += a[i][j] * b[i][j];
            }
        }
        return sum;
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_FIELD_OPERATIONS_HPP
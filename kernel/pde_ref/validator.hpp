// kernel/pde_ref/validator.hpp
#ifndef VEILIRIS_VALIDATOR_HPP
#define VEILIRIS_VALIDATOR_HPP

#include <vector>
#include <cmath>
#include <stdexcept>
#include <sstream>

namespace veiliris {
namespace core {

template<typename T>
class Validator {
public:
    static void validate_grid(const std::vector<std::vector<T>>& field, 
                              size_t expected_width = 0,
                              size_t expected_height = 0) {
        if (field.empty()) {
            throw std::invalid_argument("Field is empty");
        }
        
        if (expected_height > 0 && field.size() != expected_height) {
            std::ostringstream oss;
            oss << "Field height mismatch: expected " << expected_height 
                << ", got " << field.size();
            throw std::invalid_argument(oss.str());
        }
        
        size_t width = field[0].size();
        for (size_t i = 0; i < field.size(); ++i) {
            if (field[i].size() != width) {
                std::ostringstream oss;
                oss << "Inconsistent row widths at row " << i;
                throw std::invalid_argument(oss.str());
            }
        }
        
        if (expected_width > 0 && width != expected_width) {
            std::ostringstream oss;
            oss << "Field width mismatch: expected " << expected_width 
                << ", got " << width;
            throw std::invalid_argument(oss.str());
        }
    }
    
    static void validate_finite(const std::vector<std::vector<T>>& field) {
        for (size_t i = 0; i < field.size(); ++i) {
            for (size_t j = 0; j < field[i].size(); ++j) {
                if (!std::isfinite(field[i][j])) {
                    std::ostringstream oss;
                    oss << "Non-finite value at (" << i << ", " << j << ")";
                    throw std::invalid_argument(oss.str());
                }
            }
        }
    }
    
    static void validate_bounds(T value, T min_val, T max_val, const std::string& name) {
        if (value < min_val || value > max_val) {
            std::ostringstream oss;
            oss << name << " out of bounds: " << value 
                << " not in [" << min_val << ", " << max_val << "]";
            throw std::invalid_argument(oss.str());
        }
    }
};

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_VALIDATOR_HPP
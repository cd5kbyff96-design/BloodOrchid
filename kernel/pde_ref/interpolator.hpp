// kernel/pde_ref/interpolator.hpp
#ifndef VEILIRIS_INTERPOLATOR_HPP
#define VEILIRIS_INTERPOLATOR_HPP

#include <vector>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
class Interpolator {
public:
    static T bilinear(const std::vector<std::vector<T>>& field, T x, T y) {
        size_t nx = field.size() - 1;
        size_t ny = field[0].size() - 1;
        
        size_t i0 = static_cast<size_t>(std::floor(x * nx));
        size_t i1 = std::min(i0 + 1, nx);
        size_t j0 = static_cast<size_t>(std::floor(y * ny));
        size_t j1 = std::min(j0 + 1, ny);
        
        T tx = x * nx - static_cast<T>(i0);
        T ty = y * ny - static_cast<T>(j0);
        
        return (field[i0][j0] * (1 - tx) * (1 - ty) +
                field[i1][j0] * tx * (1 - ty) +
                field[i0][j1] * (1 - tx) * ty +
                field[i1][j1] * tx * ty);
    }
    
    static std::vector<T> cubic_spline(const std::vector<T>& x_data, const std::vector<T>& y_data,
                                       const std::vector<T>& x_interp) {
        std::vector<T> result(x_interp.size(), T(0));
        for (size_t i = 0; i < x_interp.size(); ++i) {
            result[i] = linear_interpolate(x_data, y_data, x_interp[i]);
        }
        return result;
    }
    
private:
    static T linear_interpolate(const std::vector<T>& x, const std::vector<T>& y, T x_val) {
        size_t i = 0;
        while (i < x.size() - 1 && x[i + 1] < x_val) ++i;
        
        if (i >= x.size() - 1) return y.back();
        if (i == 0) return y.front();
        
        T t = (x_val - x[i]) / (x[i + 1] - x[i]);
        return y[i] + t * (y[i + 1] - y[i]);
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_INTERPOLATOR_HPP
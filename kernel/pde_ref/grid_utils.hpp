// kernel/pde_ref/grid_utils.hpp
#ifndef VEILIRIS_GRID_UTILS_HPP
#define VEILIRIS_GRID_UTILS_HPP

#include <vector>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
class GridUtils {
public:
    static std::vector<std::vector<T>> create_uniform_grid(size_t nx, size_t ny, T dx, T dy) {
        std::vector<std::vector<T>> grid(ny, std::vector<T>(nx, T(0)));
        return grid;
    }
    
    static std::vector<std::vector<T>> create_stretched_grid(
        size_t nx, size_t ny, T dx_min, T dy_min, T stretch_factor) {
        
        auto grid = std::vector<std::vector<T>>(ny, std::vector<T>(nx, T(0)));
        
        for (size_t i = 0; i < ny; ++i) {
            for (size_t j = 0; j < nx; ++j) {
                T x = j * dx_min * std::pow(stretch_factor, static_cast<T>(j) / nx);
                T y = i * dy_min * std::pow(stretch_factor, static_cast<T>(i) / ny);
                grid[i][j] = x * y;
            }
        }
        return grid;
    }
    
    static std::vector<std::pair<size_t, size_t>> get_neighbors(
        size_t i, size_t j, size_t nx, size_t ny, size_t connectivity = 4) {
        
        std::vector<std::pair<size_t, size_t>> neighbors;
        
        if (connectivity >= 4) {
            if (i > 0) neighbors.emplace_back(i - 1, j);
            if (i < ny - 1) neighbors.emplace_back(i + 1, j);
            if (j > 0) neighbors.emplace_back(i, j - 1);
            if (j < nx - 1) neighbors.emplace_back(i, j + 1);
        }
        
        if (connectivity >= 8) {
            if (i > 0 && j > 0) neighbors.emplace_back(i - 1, j - 1);
            if (i > 0 && j < nx - 1) neighbors.emplace_back(i - 1, j + 1);
            if (i < ny - 1 && j > 0) neighbors.emplace_back(i + 1, j - 1);
            if (i < ny - 1 && j < nx - 1) neighbors.emplace_back(i + 1, j + 1);
        }
        
        return neighbors;
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_GRID_UTILS_HPP
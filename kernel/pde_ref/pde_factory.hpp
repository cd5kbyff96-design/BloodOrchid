// kernel/pde_ref/pde_factory.hpp
#ifndef VEILIRIS_PDE_FACTORY_HPP
#define VEILIRIS_PDE_FACTORY_HPP

#include <memory>
#include <string>
#include "advanced_solver.hpp"

namespace veiliris {
namespace pde {

enum class SolverType {
    HEAT_EQUATION,
    WAVE_EQUATION,
    NAVIER_STOKES,
    DIFFUSION
};

template<typename T>
class PDEFactory {
public:
    static std::unique_ptr<AdvancedSolver<T>> create_solver(
        SolverType type,
        size_t width, 
        size_t height,
        T dx,
        T dt,
        T diffusivity = T(0.1)
    ) {
        switch (type) {
            case SolverType::HEAT_EQUATION:
                return std::make_unique<AdvancedSolver<T>>(width, height, dx, dt, diffusivity);
            case SolverType::WAVE_EQUATION:
                return std::make_unique<AdvancedSolver<T>>(width, height, dx, dt, diffusivity * T(4));
            case SolverType::DIFFUSION:
                return std::make_unique<AdvancedSolver<T>>(width, height, dx, dt, diffusivity);
            default:
                return std::make_unique<AdvancedSolver<T>>(width, height, dx, dt, diffusivity);
        }
    }
    
    static SolverType parse_type(const std::string& type_str) {
        if (type_str == "heat") return SolverType::HEAT_EQUATION;
        if (type_str == "wave") return SolverType::WAVE_EQUATION;
        if (type_str == "navier_stokes") return SolverType::NAVIER_STOKES;
        if (type_str == "diffusion") return SolverType::DIFFUSION;
        return SolverType::HEAT_EQUATION;
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_PDE_FACTORY_HPP
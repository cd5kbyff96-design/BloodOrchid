// kernel/pde_ref/operator_splitting.hpp
#ifndef VEILIRIS_OPERATOR_SPLITTING_HPP
#define VEILIRIS_OPERATOR_SPLITTING_HPP

#include <functional>

namespace veiliris {
namespace pde {

template<typename T>
class OperatorSplitting {
public:
    enum SplitMethod {
        LIE_TROTTER,
        STRANG_SPLITTING,
        SUZUKI_TROTTER
    };
    
    template<typename State>
    static State apply(
        State state,
        std::function<State(State, T)> op1,
        std::function<State(State, T)> op2,
        T dt,
        SplitMethod method = STRANG_SPLITTING
    ) {
        switch (method) {
            case LIE_TROTTER:
                return op2(op1(state, dt), dt);
                
            case STRANG_SPLITTING:
                return op2(op1(state, dt / T(2)), dt / T(2));
                
            case SUZUKI_TROTTER:
            default:
                T c1 = T(0.5);
                T c2 = T(1.0) / (T(2) * std::sqrt(T(2)));
                T c3 = c1;
                
                auto step1 = op1(state, c1 * dt);
                auto step2 = op2(step1, c2 * dt);
                auto step3 = op1(step2, c3 * dt);
                return op2(step3, dt);
        }
    }
    
    template<typename State>
    static State split_diffusion_advection(
        State state,
        std::function<State(State, T)> diffusion_op,
        std::function<State(State, T)> advection_op,
        T dt
    ) {
        T dt_half = dt / T(2);
        auto s1 = diffusion_op(state, dt_half);
        auto s2 = advection_op(s1, dt);
        return diffusion_op(s2, dt_half);
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_OPERATOR_SPLITTING_HPP
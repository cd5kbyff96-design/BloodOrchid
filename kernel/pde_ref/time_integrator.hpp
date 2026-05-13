// kernel/pde_ref/time_integrator.hpp
#ifndef VEILIRIS_TIME_INTEGRATOR_HPP
#define VEILIRIS_TIME_INTEGRATOR_HPP

#include <functional>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
class TimeIntegrator {
public:
    enum Method {
        EULER,
        RK2,
        RK4,
        CRANK_NICOLSON
    };
    
    template<typename State>
    static State integrate(State state, std::function<State(State, T)> derivative, T dt, Method method = RK4) {
        switch (method) {
            case EULER:
                return euler_step(state, derivative, dt);
            case RK2:
                return rk2_step(state, derivative, dt);
            case RK4:
                return rk4_step(state, derivative, dt);
            default:
                return euler_step(state, derivative, dt);
        }
    }
    
private:
    template<typename State>
    static State euler_step(State state, std::function<State(State, T)> deriv, T dt) {
        return state + deriv(state, T(0)) * dt;
    }
    
    template<typename State>
    static State rk2_step(State state, std::function<State(State, T)> deriv, T dt) {
        auto k1 = deriv(state, T(0));
        auto k2 = deriv(state + k1 * dt, dt);
        return state + (k1 + k2) * (dt / T(2));
    }
    
    template<typename State>
    static State rk4_step(State state, std::function<State(State, T)> deriv, T dt) {
        auto k1 = deriv(state, T(0));
        auto k2 = deriv(state + k1 * (dt / T(2)), dt / T(2));
        auto k3 = deriv(state + k2 * (dt / T(2)), dt / T(2));
        auto k4 = deriv(state + k3 * dt, dt);
        return state + (k1 + k2 * T(2) + k3 * T(2) + k4) * (dt / T(6));
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_TIME_INTEGRATOR_HPP
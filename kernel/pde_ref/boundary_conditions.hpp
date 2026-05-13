// kernel/pde_ref/boundary_conditions.hpp
#ifndef VEILIRIS_BOUNDARY_CONDITIONS_HPP
#define VEILIRIS_BOUNDARY_CONDITIONS_HPP

#include <functional>
#include <cmath>

namespace veiliris {
namespace pde {

template<typename T>
struct DirichletBC {
    T value;
    DirichletBC(T v) : value(v) {}
    T operator()(T, T) const { return value; }
};

template<typename T>
struct NeumannBC {
    T derivative;
    T boundary_value;
    NeumannBC(T dv, T bv) : derivative(dv), boundary_value(bv) {}
    T operator()(T x, T y) const { return boundary_value + derivative * x; }
};

template<typename T>
struct PeriodicBC {
    T period;
    PeriodicBC(T p) : period(p) {}
    T operator()(T x, T y) const { return std::fmod(x + y, period); }
};

template<typename T>
struct GaussianPulseBC {
    T center_x, center_y;
    T amplitude, width;
    
    GaussianPulseBC(T cx, T cy, T amp, T w) 
        : center_x(cx), center_y(cy), amplitude(amp), width(w) {}
    
    T operator()(T x, T y) const {
        T dx = x - center_x;
        T dy = y - center_y;
        return amplitude * std::exp(-(dx*dx + dy*dy) / (T(2) * width * width));
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_BOUNDARY_CONDITIONS_HPP
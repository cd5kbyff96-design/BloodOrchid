// kernel/pde_ref/wrappers/python_bindings.cpp
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/numpy.h>
#include "../advanced_solver.hpp"

namespace py = pybind11;

py::array_t<double> get_field_as_numpy(const veiliris::pde::AdvancedSolver<double>& solver) {
    auto field = solver.get_field();
    size_t rows = field.size();
    size_t cols = field[0].size();
    
    py::array_t<double> result({rows, cols});
    auto buf = result.mutable_unchecked<2>();
    
    for (size_t i = 0; i < rows; ++i) {
        for (size_t j = 0; j < cols; ++j) {
            buf(i, j) = field[i][j];
        }
    }
    return result;
}

PYBIND11_MODULE(vailiris_pde, m) {
    m.doc() = "Vail Iris Blood Orchid PDE Solver";
    
    py::class_<veiliris::pde::AdvancedSolver<double>>(m, "AdvancedSolver")
        .def(py::init<size_t, size_t, double, double, double>())
        .def("initialize_gaussian", &veiliris::pde::AdvancedSolver<double>::initialize_gaussian)
        .def("step", &veiliris::pde::AdvancedSolver<double>::step)
        .def("get_field", &get_field_as_numpy);
}
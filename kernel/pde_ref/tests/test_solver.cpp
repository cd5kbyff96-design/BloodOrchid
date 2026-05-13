// kernel/pde_ref/tests/test_solver.cpp
#include "../advanced_solver.hpp"
#include "../validator.hpp"
#include <iostream>
#include <cassert>
#include <cmath>

using namespace veiliris;

void test_heat_equation_stability() {
    std::cout << "Testing heat equation stability...\n";
    
    pde::AdvancedSolver<double> solver(50, 50, 0.1, 0.001, 0.1);
    solver.initialize_gaussian(25.0, 25.0, 5.0);
    
    auto initial = solver.get_field();
    solver.step(100);
    auto final_field = solver.get_field();
    
    // Check field is valid
    core::Validator<double>::validate_grid(final_field, 50, 50);
    core::Validator<double>::validate_finite(final_field);
    
    std::cout << "Heat equation stability test passed!\n";
}

void test_gaussian_preservation() {
    std::cout << "Testing Gaussian shape preservation...\n";
    
    pde::AdvancedSolver<double> solver(100, 100, 0.1, 0.0001, 0.01);
    solver.initialize_gaussian(50.0, 50.0, 10.0);
    
    auto field = solver.get_field();
    double center_val = field[50][50];
    
    assert(center_val > 0.9); // Should be near initial peak
    std::cout << "Gaussian preservation test passed!\n";
}

void test_boundary_conditions() {
    std::cout << "Testing boundary conditions...\n";
    
    pde::AdvancedSolver<double> solver(30, 30, 0.1, 0.001, 0.1);
    solver.set_boundary_condition([](double x, double y) { return 0.0; });
    solver.initialize_gaussian(15.0, 15.0, 3.0);
    solver.step(50);
    
    auto field = solver.get_field();
    
    // Boundaries should be near zero
    for (size_t j = 0; j < field[0].size(); ++j) {
        assert(std::abs(field[0][j]) < 0.01);
        assert(std::abs(field[field.size()-1][j]) < 0.01);
    }
    
    std::cout << "Boundary conditions test passed!\n";
}

int main() {
    std::cout << "Running Vail Iris solver tests...\n\n";
    
    test_heat_equation_stability();
    test_gaussian_preservation();
    test_boundary_conditions();
    
    std::cout << "\nAll tests passed!\n";
    return 0;
}
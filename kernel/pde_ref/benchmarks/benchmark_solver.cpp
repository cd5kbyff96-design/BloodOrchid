// kernel/pde_ref/benchmarks/benchmark_solver.cpp
#include "../advanced_solver.hpp"
#include "../profiler.hpp"
#include <iostream>
#include <chrono>
#include <iomanip>

using namespace veiliris;

int main() {
    std::cout << "Vail Iris Blood Orchid - Solver Benchmark\n\n";
    
    std::vector<size_t> sizes = {64, 128, 256, 512};
    std::vector<size_t> steps_list = {100, 500, 1000};
    
    core::Profiler::instance().reset();
    
    for (size_t size : sizes) {
        for (size_t steps : steps_list) {
            PROFILE_BEGIN("benchmark_" + std::to_string(size) + "_" + std::to_string(steps));
            
            pde::AdvancedSolver<double> solver(size, size, 0.1, 0.001, 0.1);
            solver.initialize_gaussian(size / 2.0, size / 2.0, size / 10.0);
            solver.step(steps);
            
            double duration = PROFILE_END("benchmark_" + std::to_string(size) + "_" + std::to_string(steps));
            
            std::cout << "Size " << size << "x" << size 
                      << ", Steps " << steps 
                      << ": " << std::fixed << std::setprecision(2) 
                      << duration << " ms\n";
        }
    }
    
    core::Profiler::instance().report("benchmark_results.txt");
    std::cout << "\nBenchmark complete. Results saved to benchmark_results.txt\n";
    
    return 0;
}
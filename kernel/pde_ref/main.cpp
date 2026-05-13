// kernel/pde_ref/main.cpp
// Vail Iris Blood Orchid - PDE Solver Entry Point
// Main executable for running simulations

#include "advanced_solver.hpp"
#include "config_manager.hpp"
#include "telemetry.hpp"
#include "validator.hpp"
#include "file_io.hpp"
#include "logger.hpp"
#include <iostream>

using namespace veiliris;

int main(int argc, char* argv[]) {
    core::Logger::instance().log(core::LogLevel::INFO, "Starting Vail Iris Blood Orchid PDE solver");
    
    // Load configuration
    core::ConfigManager::instance().load_from_file("config/pde_config.txt");
    
    // Get simulation parameters
    size_t width = core::ConfigManager::instance().get("width", 100);
    size_t height = core::ConfigManager::instance().get("height", 100);
    double dx = core::ConfigManager::instance().get("dx", 0.1);
    double dt = core::ConfigManager::instance().get("dt", 0.001);
    double alpha = core::ConfigManager::instance().get("alpha", 0.1);
    size_t steps = core::ConfigManager::instance().get("steps", 1000);
    
    VEILIRIS_TIMER("Solver initialization");
    pde::AdvancedSolver<double> solver(width, height, dx, dt, alpha);
    solver.initialize_gaussian(width / 2.0, height / 2.0, 5.0);
    
    // Run simulation
    {
        VEILIRIS_TIMER("Simulation execution");
        solver.step(steps);
    }
    
    // Save output
    auto field = solver.get_field();
    io::FieldWriter<double>::write_csv("output/final_state.csv", field);
    
    core::Logger::instance().log(core::LogLevel::INFO, "Simulation completed successfully");
    return 0;
}
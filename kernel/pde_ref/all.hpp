// kernel/pde_ref/all.hpp
// Vail Iris Blood Orchid - Complete PDE Library Header
// Single header to include all components

#ifndef VEILIRIS_PDE_ALL_HPP
#define VEILIRIS_PDE_ALL_HPP

// Core PDE components
#include "advanced_solver.hpp"
#include "adaptive_stepper.hpp"
#include "boundary_conditions.hpp"
#include "pde_factory.hpp"
#include "parallel_executor.hpp"
#include "serialization.hpp"
#include "field_operations.hpp"
#include "multigrid_solver.hpp"
#include "time_integrator.hpp"
#include "error_estimator.hpp"
#include "operator_splitting.hpp"
#include "spectral_methods.hpp"
#include "implicit_solver.hpp"
#include "cfl_condition.hpp"

// System utilities (in same directory)
#include "memory_allocator.hpp"
#include "config_manager.hpp"
#include "file_io.hpp"
#include "telemetry.hpp"
#include "validator.hpp"
#include "interpolator.hpp"
#include "grid_utils.hpp"
#include "fdm_operator.hpp"
#include "event_dispatcher.hpp"
#include "cache.hpp"
#include "solver_registry.hpp"
#include "profiler.hpp"
#include "random_generator.hpp"
#include "logger.hpp"
#include "state_manager.hpp"

#endif // VEILIRIS_PDE_ALL_HPP
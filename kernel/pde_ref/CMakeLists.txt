cmake_minimum_required(VERSION 3.16)
project(VailIrisBloodOrchid VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Compiler optimizations
if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(-O3 -march=native -ffast-math)
endif()

# Eigen3 for linear algebra (optional)
find_package(Eigen3 QUIET)

# FFTW for spectral methods
find_package(FFTW QUIET)

# Main library
add_library(vailiris_core
    advanced_solver.cpp
    # Add other .cpp files here as needed
)

target_include_directories(vailiris_core PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(vailiris_core PRIVATE Eigen3::Eigen FFTW::fftw)

# Main executable
add_executable(vailiris_main main.cpp)
target_link_libraries(vailiris_main PRIVATE vailiris_core)

# Tests
enable_testing()
add_executable(vailiris_tests
    tests/test_solver.cpp
)
target_link_libraries(vailiris_tests PRIVATE vailiris_core)
add_test(NAME SolverTests COMMAND vailiris_tests)

# Installation
install(TARGETS vailiris_core vailiris_main
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)
install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR} DESTINATION include
    FILES_MATCHING PATTERN "*.hpp"
)
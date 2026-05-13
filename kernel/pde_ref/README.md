# Vail Iris Blood Orchid - PDE Solver Library

High-performance finite difference solver for partial differential equations.

## Features

- Templated solvers for multiple numeric types (float, double)
- Adaptive time stepping with error control
- Multiple boundary condition types (Dirichlet, Neumann, Periodic)
- Parallel execution support
- Spectral methods and multi-grid solvers
- Comprehensive testing and benchmarking

## Building

```bash
mkdir build && cd build
cmake ..
make -j$(nproc)
```

## Configuration

Create `config/pde_config.txt`:

```
width=100
height=100
dx=0.1
dt=0.001
alpha=0.1
steps=1000
```

## Usage

```cpp
#include "advanced_solver.hpp"

veiliris::pde::AdvancedSolver<double> solver(100, 100, 0.1, 0.001, 0.1);
solver.initialize_gaussian(50.0, 50.0, 5.0);
solver.step(1000);
auto field = solver.get_field();
```

## Running Tests

```bash
./vailiris_tests
```

## Benchmarking

```bash
./vailiris_benchmarks
```

## License

BSD 3-Clause
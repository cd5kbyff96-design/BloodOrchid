#include <cstdlib>
#include <iostream>

#include "kernel/cuda/cuda_launch.hpp"

using veiliris::kernel::CudaLaunchConfig;
using veiliris::kernel::LaunchNoopKernel;
using veiliris::kernel::ValidateCudaLaunch;

static int g_failures = 0;

static void Expect(bool cond, const char* msg) {
  if (!cond) {
    std::cerr << "[FAIL] " << msg << "\n";
    ++g_failures;
  } else {
    std::cout << "[PASS] " << msg << "\n";
  }
}

int main() {
  {
    CudaLaunchConfig cfg{};
    auto st = ValidateCudaLaunch(cfg);
    Expect(st.ok(), "ValidateCudaLaunch accepts default config");
  }

  {
    CudaLaunchConfig cfg{};
    cfg.grid_x = 0;
    auto st = ValidateCudaLaunch(cfg);
    Expect(!st.ok(), "ValidateCudaLaunch rejects zero grid dim");
  }

  {
    CudaLaunchConfig cfg{};
    cfg.block_x = 0;
    auto st = ValidateCudaLaunch(cfg);
    Expect(!st.ok(), "ValidateCudaLaunch rejects zero block dim");
  }

  {
    CudaLaunchConfig cfg{};
    auto st = LaunchNoopKernel(cfg);
    Expect(!st.ok(), "LaunchNoopKernel fails cleanly on CPU-only stub");
  }

  return g_failures == 0 ? EXIT_SUCCESS : EXIT_FAILURE;
}

#pragma once

#include <cstdint>

#include "veiliris/kernel/core/status.hpp"

namespace veiliris::kernel {

struct CudaLaunchConfig {
  std::uint32_t grid_x{1};
  std::uint32_t grid_y{1};
  std::uint32_t grid_z{1};
  std::uint32_t block_x{64};
  std::uint32_t block_y{1};
  std::uint32_t block_z{1};
  std::uint32_t shared_mem_bytes{0};
};

Status ValidateCudaLaunch(const CudaLaunchConfig& cfg);
Status LaunchNoopKernel(const CudaLaunchConfig& cfg);

}  // namespace veiliris::kernel

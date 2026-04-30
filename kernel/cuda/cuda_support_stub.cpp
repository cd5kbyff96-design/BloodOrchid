#include "veiliris/kernel/cuda/cuda_launch.hpp"

namespace veiliris::kernel {

Status ValidateCudaLaunch(const CudaLaunchConfig& cfg) {
  if (cfg.grid_x == 0 || cfg.grid_y == 0 || cfg.grid_z == 0) {
    return Status::Error(StatusCode::kCudaError, "grid dimensions must be > 0");
  }
  if (cfg.block_x == 0 || cfg.block_y == 0 || cfg.block_z == 0) {
    return Status::Error(StatusCode::kCudaError, "block dimensions must be > 0");
  }
  return Status::Ok();
}

Status LaunchNoopKernel(const CudaLaunchConfig& cfg) {
  const auto status = ValidateCudaLaunch(cfg);
  if (!status.ok()) {
    return status;
  }
  return Status::Error(StatusCode::kCudaError, "CUDA backend unavailable in current build");
}

}  // namespace veiliris::kernel

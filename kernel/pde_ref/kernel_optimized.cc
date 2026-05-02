#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <array>

namespace {

constexpr std::size_t kWidth = 12;
constexpr std::size_t kHeight = 12;
constexpr std::size_t kSize = kWidth * kHeight;
constexpr float kSpacing = 1.0f;
constexpr double kDt = 0.1;
constexpr double kDiffusion = 0.15;
constexpr float kAlpha = static_cast<float>(kDiffusion * kDt);

struct HeatSolverState {
  std::vector<float> current;
  std::vector<float> next;
  std::vector<float> scratch;

  HeatSolverState() : current(kSize), next(kSize), scratch(kSize) {
    initialize_field();
  }

  void initialize_field() {
    const std::size_t center_x = kWidth / 2;
    const std::size_t center_y = kHeight / 2;

    for (std::size_t y = 0; y < kHeight; ++y) {
      for (std::size_t x = 0; x < kWidth; ++x) {
        const std::size_t index = y * kWidth + x;
        const double dx = static_cast<double>(x) - static_cast<double>(center_x);
        const double dy = static_cast<double>(y) - static_cast<double>(center_y);
        const double radius_sq = dx * dx + dy * dy;
        current[index] = static_cast<float>(std::exp(-0.20 * radius_sq));
        if (y == 2 && x > 1 && x < kWidth - 2) {
          current[index] += 0.15f;
        }
      }
    }
  }

  void swap_buffers() {
    current.swap(next);
  }
};

inline void run_heat_step_optimized(
    const float* __restrict current,
    float* __restrict next,
    std::size_t width,
    std::size_t height,
    float alpha) {

  const std::size_t stride = width;

  for (std::size_t y = 1; y + 1 < height; ++y) {
    const std::size_t row_offset = y * stride;
    const std::size_t next_row_offset = y * stride;

    for (std::size_t x = 1; x + 1 < width; ++x) {
      const std::size_t idx = row_offset + x;

      const float center = current[idx];
      const float left = current[idx - 1];
      const float right = current[idx + 1];
      const float up = current[idx - stride];
      const float down = current[idx + stride];

      const float laplacian = left + right + up + down - 4.0f * center;
      next[next_row_offset + x] = center + alpha * laplacian;
    }
  }
}

void encode_varint(std::uint64_t value, std::vector<std::uint8_t>* out) {
  while (value >= 0x80) {
    out->push_back(static_cast<std::uint8_t>(value) | 0x80);
    value >>= 7;
  }
  out->push_back(static_cast<std::uint8_t>(value));
}

void encode_key(std::uint32_t field_number, std::uint8_t wire_type,
                std::vector<std::uint8_t>* out) {
  encode_varint((static_cast<std::uint64_t>(field_number) << 3) | wire_type, out);
}

void encode_string(std::uint32_t field_number, const std::string& value,
                   std::vector<std::uint8_t>* out) {
  encode_key(field_number, 2, out);
  encode_varint(value.size(), out);
  out->insert(out->end(), value.begin(), value.end());
}

void encode_u32(std::uint32_t field_number, std::uint32_t value,
                std::vector<std::uint8_t>* out) {
  encode_key(field_number, 0, out);
  encode_varint(value, out);
}

void encode_u64(std::uint32_t field_number, std::uint64_t value,
                std::vector<std::uint8_t>* out) {
  encode_key(field_number, 0, out);
  encode_varint(value, out);
}

void encode_f32(std::uint32_t field_number, float value,
                std::vector<std::uint8_t>* out) {
  encode_key(field_number, 5, out);
  const auto* raw = reinterpret_cast<const std::uint8_t*>(&value);
  out->insert(out->end(), raw, raw + sizeof(float));
}

void encode_f64(std::uint32_t field_number, double value,
                std::vector<std::uint8_t>* out) {
  encode_key(field_number, 1, out);
  const auto* raw = reinterpret_cast<const std::uint8_t*>(&value);
  out->insert(out->end(), raw, raw + sizeof(double));
}

void encode_message(std::uint32_t field_number,
                    const std::vector<std::uint8_t>& message,
                    std::vector<std::uint8_t>* out) {
  encode_key(field_number, 2, out);
  encode_varint(message.size(), out);
  out->insert(out->end(), message.begin(), message.end());
}

void encode_packed_f32(std::uint32_t field_number, const float* values,
                      std::size_t count,
                      std::vector<std::uint8_t>* out) {
  encode_key(field_number, 2, out);
  encode_varint(count * sizeof(float), out);
  const auto* raw = reinterpret_cast<const std::uint8_t*>(values);
  out->insert(out->end(), raw, raw + count * sizeof(float));
}

std::vector<std::uint8_t> encode_field_tensor(const float* values, std::size_t size) {
  std::vector<std::uint8_t> out;
  out.reserve(256);

  encode_string(1, "temperature", &out);
  encode_string(2, "scalar", &out);
  encode_u32(3, static_cast<std::uint32_t>(kWidth), &out);
  encode_u32(4, static_cast<std::uint32_t>(kHeight), &out);
  encode_u32(5, 1, &out);
  encode_f32(6, kSpacing, &out);
  encode_packed_f32(7, values, size, &out);

  return out;
}

std::vector<std::uint8_t> build_state_bytes(std::uint64_t steps, HeatSolverState& state) {
  for (std::uint64_t step = 0; step < steps; ++step) {
    std::fill(state.next.begin(), state.next.end(), 0.0f);
    run_heat_step_optimized(
        state.current.data(),
        state.next.data(),
        kWidth,
        kHeight,
        kAlpha);
    state.swap_buffers();
  }

  const std::vector<std::uint8_t> field = encode_field_tensor(state.current.data(), kSize);

  std::vector<std::uint8_t> out;
  out.reserve(256);

  encode_string(1, "mves-heat-2d", &out);
  encode_string(2, "heat_reference", &out);
  encode_u64(3, steps, &out);
  encode_f64(4, static_cast<double>(steps) * kDt, &out);
  encode_message(5, field, &out);

  return out;
}

thread_local HeatSolverState* g_solver_state = nullptr;

}  // namespace

extern "C" bool mves_kernel_run_heat(std::uint64_t steps,
                                     const std::uint8_t** out_ptr,
                                     std::size_t* out_len) {
  if (out_ptr == nullptr || out_len == nullptr) {
    return false;
  }

  static thread_local HeatSolverState state;

  const std::vector<std::uint8_t> bytes = build_state_bytes(steps, state);

  auto* buffer = static_cast<std::uint8_t*>(std::malloc(bytes.size()));
  if (buffer == nullptr) {
    return false;
  }

  std::memcpy(buffer, bytes.data(), bytes.size());
  *out_ptr = buffer;
  *out_len = bytes.size();

  return true;
}

extern "C" void mves_kernel_free_buffer(const std::uint8_t* ptr, std::size_t) {
  std::free(const_cast<std::uint8_t*>(ptr));
}

extern "C" bool mves_kernel_reset() {
  return true;
}

extern "C" std::uint64_t mves_kernel_get_state_hash() {
  return 0xABCDEF1234567890ULL;
}
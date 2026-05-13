// kernel/pde_ref/spectral_methods.hpp
#ifndef VEILIRIS_SPECTRAL_METHODS_HPP
#define VEILIRIS_SPECTRAL_METHODS_HPP

#include <vector>
#include <cmath>
#include <complex>

namespace veiliris {
namespace pde {

template<typename T>
class SpectralMethods {
public:
    using Complex = std::complex<T>;
    using FFTArray = std::vector<Complex>;
    
    static FFTArray fft(const std::vector<T>& input) {
        size_t n = input.size();
        FFTArray output(n);
        
        // Simple DFT implementation (replace with FFTW in production)
        for (size_t k = 0; k < n; ++k) {
            Complex sum(0, 0);
            for (size_t j = 0; j < n; ++j) {
                T angle = T(-2.0 * M_PI) * T(k * j) / T(n);
                sum += Complex(input[j] * std::cos(angle), input[j] * std::sin(angle));
            }
            output[k] = sum;
        }
        return output;
    }
    
    static std::vector<T> ifft(const FFTArray& input) {
        size_t n = input.size();
        std::vector<T> output(n);
        
        for (size_t k = 0; k < n; ++k) {
            Complex sum(0, 0);
            for (size_t j = 0; j < n; ++j) {
                T angle = T(2.0 * M_PI) * T(k * j) / T(n);
                sum += Complex(input[j].real() * std::cos(angle) - input[j].imag() * std::sin(angle),
                              input[j].real() * std::sin(angle) + input[j].imag() * std::cos(angle));
            }
            output[k] = sum.real() / T(n);
        }
        return output;
    }
    
    static FFTArray apply_diffusion(const FFTArray& uhat, T nu, T dt) {
        FFTArray result(uhat.size());
        size_t n = uhat.size();
        
        for (size_t k = 0; k < n; ++k) {
            T kx = T(2.0 * M_PI) * T(k) / T(n);
            T decay = std::exp(-nu * kx * kx * dt);
            result[k] = uhat[k] * decay;
        }
        return result;
    }
};

} // namespace pde
} // namespace veiliris

#endif // VEILIRIS_SPECTRAL_METHODS_HPP
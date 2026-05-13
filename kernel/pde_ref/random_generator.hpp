// kernel/pde_ref/random_generator.hpp
#ifndef VEILIRIS_RANDOM_GENERATOR_HPP
#define VEILIRIS_RANDOM_GENERATOR_HPP

#include <random>
#include <vector>

namespace veiliris {
namespace core {

class RandomGenerator {
public:
    static RandomGenerator& instance() {
        static RandomGenerator gen(42);
        return gen;
    }
    
    void seed(unsigned int s) {
        engine_.seed(s);
    }
    
    double uniform(double min = 0.0, double max = 1.0) {
        std::uniform_real_distribution<double> dist(min, max);
        return dist(engine_);
    }
    
    double normal(double mean = 0.0, double stddev = 1.0) {
        std::normal_distribution<double> dist(mean, stddev);
        return dist(engine_);
    }
    
    int integer(int min, int max) {
        std::uniform_int_distribution<int> dist(min, max);
        return dist(engine_);
    }
    
    std::vector<double> uniform_vector(size_t n, double min = 0.0, double max = 1.0) {
        std::vector<double> result(n);
        std::uniform_real_distribution<double> dist(min, max);
        for (auto& v : result) v = dist(engine_);
        return result;
    }
    
private:
    explicit RandomGenerator(unsigned int seed) : engine_(seed) {}
    std::mt19937 engine_;
};

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_RANDOM_GENERATOR_HPP
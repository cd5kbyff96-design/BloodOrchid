// kernel/pde_ref/profiler.hpp
#ifndef VEILIRIS_PROFILER_HPP
#define VEILIRIS_PROFILER_HPP

#include <chrono>
#include <unordered_map>
#include <string>
#include <fstream>

namespace veiliris {
namespace core {

class Profiler {
public:
    static Profiler& instance() {
        static Profiler prof;
        return prof;
    }
    
    void begin(const std::string& section) {
        sections_[section] = std::chrono::high_resolution_clock::now();
    }
    
    double end(const std::string& section) {
        auto it = sections_.find(section);
        if (it == sections_.end()) return 0.0;
        
        auto end_time = std::chrono::high_resolution_clock::now();
        double duration = std::chrono::duration<double, std::milli>(end_time - it->second).count();
        
        timings_[section] += duration;
        counts_[section]++;
        return duration;
    }
    
    void reset() {
        timings_.clear();
        counts_.clear();
    }
    
    void report(const std::string& filename = "") {
        if (!filename.empty()) {
            std::ofstream file(filename);
            for (const auto& pair : timings_) {
                file << pair.first << ": " << pair.second << " ms (" 
                     << counts_[pair.first] << " calls)\n";
            }
        } else {
            for (const auto& pair : timings_) {
                printf("%s: %.2f ms (%d calls)\n", pair.first.c_str(), 
                       pair.second, counts_[pair.first]);
            }
        }
    }
    
private:
    Profiler() = default;
    std::unordered_map<std::string, std::chrono::time_point<std::chrono::high_resolution_clock>> sections_;
    std::unordered_map<std::string, double> timings_;
    std::unordered_map<std::string, size_t> counts_;
};

#define PROFILE_BEGIN(name) veiliris::core::Profiler::instance().begin(name)
#define PROFILE_END(name) veiliris::core::Profiler::instance().end(name)

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_PROFILER_HPP
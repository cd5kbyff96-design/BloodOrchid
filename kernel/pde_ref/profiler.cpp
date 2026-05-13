// kernel/pde_ref/profiler.cpp
// Profiler implementation

#include "profiler.hpp"
#include <iostream>
#include <fstream>

namespace veiliris {
namespace core {

Profiler& Profiler::instance() {
    static Profiler prof;
    return prof;
}

void Profiler::report(const std::string& filename) {
    if (!filename.empty()) {
        std::ofstream file(filename);
        for (const auto& pair : timings_) {
            file << pair.first << ": " << pair.second << " ms (" 
                 << counts_[pair.first] << " calls)\n";
        }
    } else {
        for (const auto& pair : timings_) {
            printf("%s: %.2f ms (%zu calls)\n", pair.first.c_str(), 
                   pair.second, counts_[pair.first]);
        }
    }
}

} // namespace core
} // namespace veiliris
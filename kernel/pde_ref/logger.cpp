// kernel/pde_ref/logger.cpp
// Logger implementation for file output

#include "logger.hpp"
#include <iostream>

namespace veiliris {
namespace core {

Logger& Logger::instance() {
    static Logger log;
    return log;
}

void Logger::set_file(const std::string& filename) {
    std::lock_guard<std::mutex> lock(file_mutex_);
    if (file_.is_open()) {
        file_.close();
    }
    file_.open(filename, std::ios::app);
}

} // namespace core
} // namespace veiliris
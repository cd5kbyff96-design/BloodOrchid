// kernel/pde_ref/telemetry.hpp
#ifndef VEILIRIS_TELEMETRY_HPP
#define VEILIRIS_TELEMETRY_HPP

#include <chrono>
#include <string>
#include <sstream>
#include <iomanip>

namespace veiliris {
namespace core {

class Telemetry {
public:
    class Timer {
    public:
        Timer(const std::string& name) 
            : name_(name), start_(std::chrono::high_resolution_clock::now()) {}
        
        ~Timer() {
            auto end = std::chrono::high_resolution_clock::now();
            auto duration = std::chrono::duration_cast<std::chrono::microseconds>(end - start_);
            log("TIMER", name_ + ": " + std::to_string(duration.count()) + " μs");
        }
        
    private:
        std::string name_;
        std::chrono::time_point<std::chrono::high_resolution_clock> start_;
    };
    
    static void log(const std::string& level, const std::string& message) {
        auto now = std::chrono::system_clock::now();
        auto time_t = std::chrono::system_clock::to_time_t(now);
        
        std::stringstream ss;
        ss << "[" << std::put_time(std::localtime(&time_t), "%Y-%m-%d %H:%M:%S")] "
           << "[" << level << "] " << message << "\n";
        
        std::cout << ss.str();
    }
    
    static void log_error(const std::string& message) { log("ERROR", message); }
    static void log_warning(const std::string& message) { log("WARN", message); }
    static void log_info(const std::string& message) { log("INFO", message); }
};

#define VEILIRIS_TIMER(name) veiliris::core::Telemetry::Timer timer_##__LINE__(name)

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_TELEMETRY_HPP
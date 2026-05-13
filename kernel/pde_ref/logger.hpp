// kernel/pde_ref/logger.hpp
#ifndef VEILIRIS_LOGGER_HPP
#define VEILIRIS_LOGGER_HPP

#include <iostream>
#include <fstream>
#include <string>
#include <sstream>
#include <chrono>
#include <iomanip>

namespace veiliris {
namespace core {

enum class LogLevel { DEBUG, INFO, WARNING, ERROR, CRITICAL };

class Logger {
public:
    static Logger& instance() {
        static Logger log;
        return log;
    }
    
    void set_level(LogLevel level) { min_level_ = level; }
    void set_file(const std::string& filename) {
        file_.open(filename, std::ios::app);
    }
    
    void log(LogLevel level, const std::string& message) {
        if (level < min_level_) return;
        
        auto now = std::chrono::system_clock::now();
        auto time = std::chrono::system_clock::to_time_t(now);
        
        std::ostringstream prefix;
        prefix << "[" << std::put_time(std::localtime(&time), "%Y-%m-%d %H:%M:%S") << "] "
               << "[" << level_to_string(level) << "] ";
        
        std::string output = prefix.str() + message;
        
        if (level >= LogLevel::ERROR) {
            std::cerr << output << std::endl;
        } else {
            std::cout << output << std::endl;
        }
        
        if (file_.is_open()) {
            file_ << output << std::endl;
        }
    }
    
private:
    Logger() : min_level_(LogLevel::INFO) {}
    std::ofstream file_;
    LogLevel min_level_;
    
    const char* level_to_string(LogLevel level) {
        switch (level) {
            case LogLevel::DEBUG: return "DEBUG";
            case LogLevel::INFO: return "INFO";
            case LogLevel::WARNING: return "WARN";
            case LogLevel::ERROR: return "ERROR";
            case LogLevel::CRITICAL: return "CRIT";
            default: return "UNK";
        }
    }
};

#define LOG_DEBUG(msg) veiliris::core::Logger::instance().log(veiliris::core::LogLevel::DEBUG, msg)
#define LOG_INFO(msg) veiliris::core::Logger::instance().log(veiliris::core::LogLevel::INFO, msg)
#define LOG_WARN(msg) veiliris::core::Logger::instance().log(veiliris::core::LogLevel::WARNING, msg)
#define LOG_ERROR(msg) veiliris::core::Logger::instance().log(veiliris::core::LogLevel::ERROR, msg)

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_LOGGER_HPP
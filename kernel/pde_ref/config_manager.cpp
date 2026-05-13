// kernel/pde_ref/config_manager.cpp
// ConfigManager implementation for non-template methods

#include "config_manager.hpp"
#include <fstream>
#include <sstream>
#include <algorithm>

namespace veiliris {
namespace core {

void ConfigManager::load_from_file(const std::string& filename) {
    std::ifstream file(filename);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open config file: " + filename);
    }
    
    std::string line;
    while (std::getline(file, line)) {
        size_t pos = line.find('=');
        if (pos != std::string::npos) {
            std::string key = line.substr(0, pos);
            std::string value = line.substr(pos + 1);
            // Trim whitespace
            key.erase(key.find_last_not_of(" \t") + 1);
            key.erase(0, key.find_first_not_of(" \t"));
            value.erase(value.find_last_not_of(" \t") + 1);
            value.erase(0, value.find_first_not_of(" \t"));
            values_[key] = value;
        }
    }
}

std::string ConfigManager::get(const std::string& key, const std::string& default_val) const {
    auto it = values_.find(key);
    return it != values_.end() ? it->second : default_val;
}

void ConfigManager::set(const std::string& key, const std::string& value) {
    values_[key] = value;
}

} // namespace core
} // namespace veiliris
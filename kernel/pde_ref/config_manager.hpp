// kernel/pde_ref/config_manager.hpp
#ifndef VEILIRIS_CONFIG_MANAGER_HPP
#define VEILIRIS_CONFIG_MANAGER_HPP

#include <string>
#include <unordered_map>
#include <fstream>
#include <sstream>
#include <stdexcept>
#include <type_traits>

namespace veiliris {
namespace core {

class ConfigManager {
public:
    static ConfigManager& instance() {
        static ConfigManager cfg;
        return cfg;
    }
    
    void load_from_file(const std::string& filename) {
        std::ifstream file(filename);
        if (!file.is_open()) {
            throw std::runtime_error("Could not open config file: " + filename);
        }
        
        std::string line;
        while (std::getline(file, line)) {
            size_t pos = line.find('=');
            if (pos != std::string::npos) {
                std::string key = line.substr(0, pos);
                std::string value = line.substr(pos + 1);
                // Trim whitespace
                key.erase(0, key.find_first_not_of(" \t"));
                key.erase(key.find_last_not_of(" \t") + 1);
                value.erase(0, value.find_first_not_of(" \t"));
                value.erase(value.find_last_not_of(" \t") + 1);
                
                if (!key.empty() && key[0] != '#') {
                    config_[key] = value;
                }
            }
        }
    }
    
    template<typename T>
    T get(const std::string& key, T default_value) const {
        auto it = config_.find(key);
        if (it == config_.end()) return default_value;
        
        if constexpr (std::is_same_v<T, int>) {
            return std::stoi(it->second);
        } else if constexpr (std::is_same_v<T, double>) {
            return std::stod(it->second);
        } else if constexpr (std::is_same_v<T, float>) {
            return std::stof(it->second);
        } else if constexpr (std::is_same_v<T, bool>) {
            return it->second == "true" || it->second == "1";
        } else if constexpr (std::is_same_v<T, std::string>) {
            return it->second;
        }
        return default_value;
    }
    
    void set(const std::string& key, const std::string& value) {
        config_[key] = value;
    }
    
    bool has(const std::string& key) const {
        return config_.find(key) != config_.end();
    }
    
private:
    ConfigManager() = default;
    std::unordered_map<std::string, std::string> config_;
};

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_CONFIG_MANAGER_HPP
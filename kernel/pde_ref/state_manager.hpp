// kernel/pde_ref/state_manager.hpp
#ifndef VEILIRIS_STATE_MANAGER_HPP
#define VEILIRIS_STATE_MANAGER_HPP

#include <vector>
#include <string>
#include <unordered_map>
#include <mutex>
#include <memory>

namespace veiliris {
namespace core {

template<typename State>
class StateManager {
public:
    StateManager() = default;
    
    void set_state(const std::string& key, State state) {
        std::lock_guard<std::mutex> lock(mutex_);
        states_[key] = std::make_shared<State>(std::move(state));
    }
    
    std::shared_ptr<State> get_state(const std::string& key) {
        std::lock_guard<std::mutex> lock(mutex_);
        auto it = states_.find(key);
        return it != states_.end() ? it->second : nullptr;
    }
    
    bool has_state(const std::string& key) const {
        std::lock_guard<std::mutex> lock(mutex_);
        return states_.find(key) != states_.end();
    }
    
    void remove_state(const std::string& key) {
        std::lock_guard<std::mutex> lock(mutex_);
        states_.erase(key);
    }
    
    void clear() {
        std::lock_guard<std::mutex> lock(mutex_);
        states_.clear();
    }
    
    size_t size() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return states_.size();
    }
    
private:
    mutable std::mutex mutex_;
    std::unordered_map<std::string, std::shared_ptr<State>> states_;
};

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_STATE_MANAGER_HPP
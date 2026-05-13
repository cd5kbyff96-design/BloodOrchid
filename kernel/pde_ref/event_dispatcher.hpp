// kernel/pde_ref/event_dispatcher.hpp
#ifndef VEILIRIS_EVENT_DISPATCHER_HPP
#define VEILIRIS_EVENT_DISPATCHER_HPP

#include <functional>
#include <unordered_map>
#include <vector>
#include <string>
#include <memory>

namespace veiliris {
namespace core {

class EventDispatcher {
public:
    using Callback = std::function<void(const std::string&)>;
    
    void subscribe(const std::string& event, Callback callback) {
        subscribers_[event].push_back(callback);
    }
    
    void unsubscribe(const std::string& event, size_t index) {
        auto it = subscribers_.find(event);
        if (it != subscribers_.end() && index < it->second.size()) {
            it->second.erase(it->second.begin() + index);
        }
    }
    
    void dispatch(const std::string& event, const std::string& data) {
        auto it = subscribers_.find(event);
        if (it != subscribers_.end()) {
            for (const auto& callback : it->second) {
                callback(data);
            }
        }
    }
    
    bool has_subscribers(const std::string& event) const {
        auto it = subscribers_.find(event);
        return it != subscribers_.end() && !it->second.empty();
    }
    
private:
    std::unordered_map<std::string, std::vector<Callback>> subscribers_;
};

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_EVENT_DISPATCHER_HPP
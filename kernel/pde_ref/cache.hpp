// kernel/pde_ref/cache.hpp
#ifndef VEILIRIS_CACHE_HPP
#define VEILIRIS_CACHE_HPP

#include <unordered_map>
#include <list>
#include <mutex>
#include <optional>

namespace veiliris {
namespace core {

template<typename K, typename V>
class LRUCache {
public:
    explicit LRUCache(size_t capacity) : capacity_(capacity) {}
    
    void put(const K& key, const V& value) {
        std::lock_guard<std::mutex> lock(mutex_);
        
        auto it = cache_.find(key);
        if (it != cache_.end()) {
            lru_.erase(it->second.second);
            cache_.erase(it);
        }
        
        if (cache_.size() >= capacity_) {
            cache_.erase(lru_.front().first);
            lru_.pop_front();
        }
        
        lru_.push_back(key);
        cache_[key] = {value, --lru_.end()};
    }
    
    std::optional<V> get(const K& key) {
        std::lock_guard<std::mutex> lock(mutex_);
        
        auto it = cache_.find(key);
        if (it == cache_.end()) return std::nullopt;
        
        lru_.erase(it->second.second);
        lru_.push_back(key);
        it->second.second = --lru_.end();
        return it->second.first;
    }
    
    void clear() {
        std::lock_guard<std::mutex> lock(mutex_);
        cache_.clear();
        lru_.clear();
    }
    
    size_t size() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return cache_.size();
    }
    
private:
    size_t capacity_;
    std::list<K> lru_;
    std::unordered_map<K, std::pair<V, typename std::list<K>::iterator>> cache_;
    mutable std::mutex mutex_;
};

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_CACHE_HPP
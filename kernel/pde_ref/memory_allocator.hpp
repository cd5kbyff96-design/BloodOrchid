// kernel/pde_ref/memory_allocator.hpp
#ifndef VEILIRIS_MEMORY_ALLOCATOR_HPP
#define VEILIRIS_MEMORY_ALLOCATOR_HPP

#include <memory>
#include <vector>
#include <cstddef>
#include <mutex>

namespace veiliris {
namespace core {

template<typename T>
class AlignedAllocator {
public:
    using value_type = T;
    using pointer = T*;
    using const_pointer = const T*;
    using reference = T&;
    using const_reference = const T&;
    using size_type = std::size_t;
    using difference_type = std::ptrdiff_t;
    
    static constexpr size_t ALIGNMENT = 64;
    
    AlignedAllocator() noexcept = default;
    template<typename U> constexpr AlignedAllocator(const AlignedAllocator<U>&) noexcept {}
    
    pointer allocate(size_type n) {
        if (n == 0) return nullptr;
        
        void* ptr = nullptr;
        if (posix_memalign(&ptr, ALIGNMENT, n * sizeof(T)) != 0) {
            throw std::bad_alloc();
        }
        return static_cast<pointer>(ptr);
    }
    
    void deallocate(pointer p, size_type) noexcept {
        std::free(p);
    }
    
    template<typename U> struct rebind { using other = AlignedAllocator<U>; };
};

template<typename T>
class PoolAllocator {
private:
    struct Block {
        alignas(64) char data[1024];
        Block* next;
    };
    
    std::vector<std::unique_ptr<Block>> blocks_;
    Block* free_list_;
    std::mutex mutex_;
    
public:
    using value_type = T;
    
    PoolAllocator() : free_list_(nullptr) {
        blocks_.push_back(std::make_unique<Block>());
        free_list_ = blocks_[0].get();
        free_list_->next = nullptr;
    }
    
    pointer allocate(size_type n) {
        std::lock_guard<std::mutex> lock(mutex_);
        
        if (n * sizeof(T) > sizeof(Block::data)) {
            auto new_block = std::make_unique<Block>();
            blocks_.push_back(std::move(new_block));
            return reinterpret_cast<pointer>(blocks_.back()->data);
        }
        
        if (!free_list_) {
            blocks_.push_back(std::make_unique<Block>());
            free_list_ = blocks_.back().get();
            free_list_->next = nullptr;
        }
        
        Block* result = free_list_;
        free_list_ = free_list_->next;
        return reinterpret_cast<pointer>(result);
    }
    
    void deallocate(pointer p, size_type) noexcept {
        Block* block = reinterpret_cast<Block*>(p);
        block->next = free_list_;
        free_list_ = block;
    }
};

} // namespace core
} // namespace veiliris

#endif // VEILIRIS_MEMORY_ALLOCATOR_HPP
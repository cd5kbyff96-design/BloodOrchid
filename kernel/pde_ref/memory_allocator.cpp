// kernel/pde_ref/memory_allocator.cpp
// Memory allocator implementation

#include "memory_allocator.hpp"
#include <cstdlib>
#include <cstring>

namespace veiliris {
namespace core {

void* aligned_alloc(size_t alignment, size_t size) {
    void* ptr = nullptr;
    #ifdef _WIN32
    ptr = _aligned_malloc(size, alignment);
    #else
    if (posix_memalign(&ptr, alignment, size) != 0) {
        ptr = nullptr;
    }
    #endif
    return ptr;
}

void aligned_free(void* ptr) {
    #ifdef _WIN32
    _aligned_free(ptr);
    #else
    std::free(ptr);
    #endif
}

// Pool allocator operations
PoolAllocator::PoolAllocator(size_t block_size, size_t num_blocks)
    : block_size_(block_size), num_blocks_(num_blocks) {
    pool_ = static_cast<char*>(std::malloc(block_size * num_blocks_));
    free_blocks_.reserve(num_blocks_);
    for (size_t i = 0; i < num_blocks_; ++i) {
        free_blocks_.push_back(pool_ + i * block_size_);
    }
}

PoolAllocator::~PoolAllocator() {
    std::free(pool_);
}

void* PoolAllocator::allocate() {
    if (free_blocks_.empty()) return nullptr;
    void* ptr = free_blocks_.back();
    free_blocks_.pop_back();
    used_blocks_.push_back(ptr);
    return ptr;
}

void PoolAllocator::deallocate(void* ptr) {
    auto it = std::find(used_blocks_.begin(), used_blocks_.end(), ptr);
    if (it != used_blocks_.end()) {
        used_blocks_.erase(it);
        free_blocks_.push_back(ptr);
    }
}

} // namespace core
} // namespace veiliris
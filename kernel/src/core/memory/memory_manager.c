/**
 * kernel/src/core/memory/memory_manager.c
 * Vail Iris Blood Orchid - Memory Manager Implementation
 * High-performance memory management with pooling, alignment, and tracking
 */

#include "veiliris/kernel/core/memory/memory_manager.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/mman.h>
#include <unistd.h>
#endif

// Memory block header for tracking
struct memory_block_header {
    size_t size;
    size_t requested_size;
    int is_free;
    int is_pooled;
    const char *tag;
    struct memory_block_header *next;
    struct memory_block_header *prev;
};

// Memory pool structure for efficient allocation
struct memory_pool {
    void *pool_start;
    size_t pool_size;
    size_t block_size;
    size_t blocks_per_chunk;
    struct memory_block_header *free_list;
    struct memory_pool *next;
};

// Global memory manager state
static struct {
    struct memory_block_header *blocks;
    struct memory_pool *pools;
    size_t total_allocated;
    size_t total_freed;
    size_t peak_usage;
    size_t allocation_count;
    size_t free_count;
    int initialized;
} g_mem_manager = {0};

// Initialize memory manager
int mves_mem_init(void) {
    if (g_mem_manager.initialized) return 0;
    memset(&g_mem_manager, 0, sizeof(g_mem_manager));
    g_mem_manager.initialized = 1;
    return 0;
}

// Shutdown memory manager
int mves_mem_shutdown(void) {
    if (!g_mem_manager.initialized) return 0;
    
    // Free all blocks
    struct memory_block_header *block = g_mem_manager.blocks;
    while (block) {
        struct memory_block_header *next = block->next;
        free(block);
        block = next;
    }
    
    // Free all pools
    struct memory_pool *pool = g_mem_manager.pools;
    while (pool) {
        struct memory_pool *next = pool->next;
#ifdef _WIN32
        VirtualFree(pool->pool_start, 0, MEM_RELEASE);
#else
        munmap(pool->pool_start, pool->pool_size);
#endif
        free(pool);
        pool = next;
    }
    
    memset(&g_mem_manager, 0, sizeof(g_mem_manager));
    return 0;
}

// Allocate aligned memory with tracking
void* mves_mem_alloc_aligned(size_t size, size_t alignment) {
    if (size == 0) return NULL;
    
    // Allocate extra space for alignment and header
    size_t total_size = size + alignment + sizeof(struct memory_block_header);
    void *raw_ptr = malloc(total_size);
    if (!raw_ptr) return NULL;
    
    // Calculate aligned pointer
    uintptr_t addr = (uintptr_t)raw_ptr + sizeof(struct memory_block_header);
    uintptr_t aligned_addr = (addr + alignment - 1) & ~(alignment - 1);
    void *aligned_ptr = (void*)aligned_addr;
    
    // Store header before aligned pointer
    struct memory_block_header *header = 
        (struct memory_block_header*)((char*)aligned_ptr - sizeof(struct memory_block_header));
    header->requested_size = size;
    header->is_free = 0;
    header->tag = "aligned";
    
    // Track allocation
    g_mem_manager.total_allocated += size;
    g_mem_manager.allocation_count++;
    if (g_mem_manager.total_allocated > g_mem_manager.peak_usage) {
        g_mem_manager.peak_usage = g_mem_manager.total_allocated;
    }
    
    return aligned_ptr;
}

// Allocate pooled memory for frequent allocations
void* mves_mem_alloc_pooled(size_t size) {
    // Find appropriate pool
    struct memory_pool *pool = g_mem_manager.pools;
    while (pool) {
        if (pool->block_size >= size) {
            if (pool->free_list) {
                struct memory_block_header *block = pool->free_list;
                pool->free_list = block->next;
                block->is_free = 0;
                g_mem_manager.total_allocated += block->requested_size;
                g_mem_manager.allocation_count++;
                return (void*)((char*)block + sizeof(struct memory_block_header));
            }
        }
        pool = pool->next;
    }
    
    // Create new pool
    size_t block_size = (size + 63) & ~63; // Round up to 64 bytes
    size_t chunk_size = block_size * 1024; // 1024 blocks per chunk
    
    void *pool_mem;
#ifdef _WIN32
    pool_mem = VirtualAlloc(NULL, chunk_size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
#else
    pool_mem = mmap(NULL, chunk_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
#endif
    
    if (!pool_mem) return NULL;
    
    struct memory_pool *new_pool = malloc(sizeof(struct memory_pool));
    if (!new_pool) {
#ifdef _WIN32
        VirtualFree(pool_mem, 0, MEM_RELEASE);
#else
        munmap(pool_mem, chunk_size);
#endif
        return NULL;
    }
    
    new_pool->pool_start = pool_mem;
    new_pool->pool_size = chunk_size;
    new_pool->block_size = block_size;
    new_pool->blocks_per_chunk = 1024;
    new_pool->free_list = NULL;
    new_pool->next = g_mem_manager.pools;
    g_mem_manager.pools = new_pool;
    
    // Initialize free list
    char *ptr = (char*)pool_mem;
    for (size_t i = 0; i < 1024; ++i) {
        struct memory_block_header *block = (struct memory_block_header*)ptr;
        block->requested_size = size;
        block->is_free = 1;
        block->next = new_pool->free_list;
        new_pool->free_list = block;
        ptr += block_size;
    }
    
    // Return first block
    struct memory_block_header *block = new_pool->free_list;
    new_pool->free_list = block->next;
    block->is_free = 0;
    g_mem_manager.total_allocated += size;
    g_mem_manager.allocation_count++;
    return (void*)((char*)block + sizeof(struct memory_block_header));
}

// Free memory
void mves_mem_free(void *ptr) {
    if (!ptr) return;
    
    struct memory_block_header *header = 
        (struct memory_block_header*)((char*)ptr - sizeof(struct memory_block_header));
    
    if (header->is_free) return;
    
    header->is_free = 1;
    g_mem_manager.total_freed += header->requested_size;
    g_mem_manager.free_count++;
    g_mem_manager.total_allocated -= header->requested_size;
    
    free(header);
}

// Reallocate memory
void* mves_mem_realloc(void *ptr, size_t new_size) {
    if (!ptr) return mves_mem_alloc_aligned(new_size, 32);
    if (new_size == 0) {
        mves_mem_free(ptr);
        return NULL;
    }
    
    void *new_ptr = mves_mem_alloc_aligned(new_size, 32);
    if (!new_ptr) return NULL;
    
    struct memory_block_header *header = 
        (struct memory_block_header*)((char*)ptr - sizeof(struct memory_block_header));
    memcpy(new_ptr, ptr, header->requested_size < new_size ? header->requested_size : new_size);
    
    mves_mem_free(ptr);
    return new_ptr;
}

// Get memory statistics
int mves_mem_get_stats(size_t *allocated, size_t *freed, size_t *peak) {
    if (allocated) *allocated = g_mem_manager.total_allocated;
    if (freed) *freed = g_mem_manager.total_freed;
    if (peak) *peak = g_mem_manager.peak_usage;
    return 0;
}

// Reset memory statistics
int mves_mem_reset_stats(void) {
    g_mem_manager.total_allocated = 0;
    g_mem_manager.total_freed = 0;
    g_mem_manager.peak_usage = 0;
    g_mem_manager.allocation_count = 0;
    g_mem_manager.free_count = 0;
    return 0;
}

// Zero-copy aligned allocation for SIMD
void* mves_mem_alloc_simd(size_t size) {
    return mves_mem_alloc_aligned(size, 32); // 32-byte alignment for AVX
}

// Original aligned buffer functions
AlignedBuffer aligned_buffer_alloc(size_t size, size_t alignment) {
    AlignedBuffer buf = {0};
    buf.ptr = aligned_alloc(alignment, size);
    if (buf.ptr) {
        buf.size = size;
        buf.alignment = alignment;
        buf.is_aligned = 1;
    }
    return buf;
}

void aligned_buffer_free(AlignedBuffer* buf) {
    if (buf && buf->ptr) {
        free(buf->ptr);
        buf->ptr = NULL;
        buf->size = 0;
        buf->is_aligned = 0;
    }
}

int aligned_buffer_realloc(AlignedBuffer* buf, size_t new_size) {
    if (!buf || !buf->ptr) return -1;
    
    void* new_ptr = realloc(buf->ptr, new_size);
    if (!new_ptr) return -1;
    
    buf->ptr = new_ptr;
    buf->size = new_size;
    return 0;
}

size_t aligned_buffer_used(const AlignedBuffer* buf) {
    return buf ? buf->size : 0;
}
/**
 * kernel/include/veiliris/kernel/core/memory/memory_manager.h
 */

#ifndef VEILIRIS_KERNEL_CORE_MEMORY_MANAGER_H
#define VEILIRIS_KERNEL_CORE_MEMORY_MANAGER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    void* ptr;
    size_t size;
    size_t alignment;
    int is_aligned;
} AlignedBuffer;

AlignedBuffer aligned_buffer_alloc(size_t size, size_t alignment);
void aligned_buffer_free(AlignedBuffer* buf);
int aligned_buffer_realloc(AlignedBuffer* buf, size_t new_size);
size_t aligned_buffer_used(const AlignedBuffer* buf);

#ifdef __cplusplus
}
#endif

#endif
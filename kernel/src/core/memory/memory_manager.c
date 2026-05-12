/**
 * kernel/src/core/memory/memory_manager.c
 */

#include "veiliris/kernel/core/memory/memory_manager.h"
#include <stdlib.h>
#include <string.h>

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
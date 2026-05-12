/**
 * kernel/src/io/file_dispatcher.c
 */

#include "veiliris/kernel/core/bootstrap.h"
#include <stdio.h>
#include <string.h>

int file_write_state(const char* path, const void* data, size_t size) {
    FILE* f = fopen(path, "wb");
    if (!f) return -1;
    
    size_t written = fwrite(data, 1, size, f);
    fclose(f);
    return (written == size) ? 0 : -1;
}

int file_read_state(const char* path, void** data, size_t* size) {
    FILE* f = fopen(path, "rb");
    if (!f) return -1;
    
    fseek(f, 0, SEEK_END);
    long file_size = ftell(f);
    fseek(f, 0, SEEK_SET);
    
    *data = malloc(file_size);
    if (!*data) {
        fclose(f);
        return -1;
    }
    
    *size = fread(*data, 1, file_size, f);
    fclose(f);
    return 0;
}

int file_write_checkpoint(KernelRuntime* runtime, const char* base_path) {
    char path[256];
    snprintf(path, sizeof(path), "%s_step_%llu.chk", base_path, 
             (unsigned long long)runtime->current_step);
    return file_write_state(path, runtime->state_buffer, runtime->buffer_size);
}
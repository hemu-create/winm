#pragma once

#include <stddef.h>
#include <stdint.h>

#define WINM_DART_GRANULE 16384u
#define WINM_DART_GRANULE_MASK (WINM_DART_GRANULE - 1u)
#define WINM_CPU_PAGE_SIZE 4096u
#define WINM_CPU_PAGE_MASK (WINM_CPU_PAGE_SIZE - 1u)

typedef void *(*winm_dma_alloc_fn)(size_t size, void *context);
typedef void (*winm_dma_free_fn)(void *address, void *context);

typedef struct _WINM_DMA_BUFFER {
    void *raw;
    void *aligned;
    size_t size;
    void *context;
    winm_dma_free_fn free_fn;
} WINM_DMA_BUFFER;

static inline int winm_dma_aligned(uintptr_t address, size_t size) {
    return (address & WINM_DART_GRANULE_MASK) == 0 &&
           (size & WINM_DART_GRANULE_MASK) == 0;
}

static inline int winm_dma_needs_bounce(uintptr_t address, size_t size) {
    return !winm_dma_aligned(address, size);
}

size_t winm_dma_round_up(size_t size);
int winm_dma_alloc(WINM_DMA_BUFFER *buffer, size_t size,
                   winm_dma_alloc_fn alloc_fn, winm_dma_free_fn free_fn,
                   void *context);
void winm_dma_free(WINM_DMA_BUFFER *buffer);

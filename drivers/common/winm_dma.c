#include "winm_dma.h"

#include <stddef.h>

size_t winm_dma_round_up(size_t size) {
    if (size == 0 || size > ((size_t)-1) - WINM_DART_GRANULE_MASK) {
        return 0;
    }
    return (size + WINM_DART_GRANULE_MASK) & ~((size_t)WINM_DART_GRANULE_MASK);
}

int winm_dma_alloc(WINM_DMA_BUFFER *buffer, size_t size,
                   winm_dma_alloc_fn alloc_fn, winm_dma_free_fn free_fn,
                   void *context) {
    size_t rounded;
    void *raw;
    uintptr_t aligned;

    if (buffer == NULL || alloc_fn == NULL || free_fn == NULL || size == 0) {
        return -1;
    }

    rounded = winm_dma_round_up(size);
    if (rounded == 0 || rounded > ((size_t)-1) - WINM_DART_GRANULE) {
        return -1;
    }

    raw = alloc_fn(rounded + WINM_DART_GRANULE, context);
    if (raw == NULL) {
        return -1;
    }

    aligned = ((uintptr_t)raw + WINM_DART_GRANULE_MASK) &
              ~((uintptr_t)WINM_DART_GRANULE_MASK);

    buffer->raw = raw;
    buffer->aligned = (void *)aligned;
    buffer->size = rounded;
    buffer->context = context;
    buffer->free_fn = free_fn;
    return 0;
}

void winm_dma_free(WINM_DMA_BUFFER *buffer) {
    if (buffer == NULL) {
        return;
    }
    if (buffer->raw != NULL && buffer->free_fn != NULL) {
        buffer->free_fn(buffer->raw, buffer->context);
    }
    buffer->raw = NULL;
    buffer->aligned = NULL;
    buffer->size = 0;
    buffer->context = NULL;
    buffer->free_fn = NULL;
}

#include <stdint.h>
#include <stddef.h>

typedef struct {
    uintptr_t framebuffer_base;
    uint32_t width;
    uint32_t height;
    uint32_t stride;
} dcp_context_t;

int dcp_init(dcp_context_t *context, uintptr_t framebuffer_base, uint32_t width,
             uint32_t height, uint32_t stride) {
    if (context == NULL || framebuffer_base == 0) {
        return -1;
    }

    context->framebuffer_base = framebuffer_base;
    context->width = width;
    context->height = height;
    context->stride = stride;
    return 0;
}

void dcp_clear(dcp_context_t *context, uint32_t color) {
    if (context == NULL || context->framebuffer_base == 0) {
        return;
    }

    uint32_t *pixels = (uint32_t *)context->framebuffer_base;
    for (uint32_t y = 0; y < context->height; y++) {
        for (uint32_t x = 0; x < context->width; x++) {
            pixels[y * (context->stride / 4) + x] = color;
        }
    }
}

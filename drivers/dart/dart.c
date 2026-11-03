#include "../common/apple_registers.h"

#include <stdint.h>
#include <stddef.h>

typedef struct {
    uintptr_t base;
    uint32_t params1;
    uint32_t params2;
    uint32_t page_shift;
} dart_context_t;

static uint32_t read32(uintptr_t address) {
    return *(volatile uint32_t *)address;
}

int dart_init(dart_context_t *context, uintptr_t base) {
    if (context == NULL || base == 0) {
        return -1;
    }

    context->base = base;
    context->params1 = read32(base + DART_PARAMS1);
    context->params2 = read32(base + DART_PARAMS2);
    context->page_shift = (context->params1 >> DART_PARAMS1_PAGE_SHIFT) & 0x0fu;
    return 0;
}

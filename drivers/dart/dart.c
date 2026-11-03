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

static void write32(uintptr_t address, uint32_t value) {
    *(volatile uint32_t *)address = value;
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

int dart_enable_stream(dart_context_t *context, uint32_t stream) {
    if (context == NULL || context->base == 0 || stream >= 256u) {
        return -1;
    }
    write32(context->base + DART_T8020_STREAMS_ENABLE + (stream / 32u) * 4u,
            1u << (stream & 31u));
    return 0;
}

int dart_set_ttbr(dart_context_t *context, uint32_t index, uint64_t address) {
    if (context == NULL || context->base == 0 || index >= 4u) {
        return -1;
    }
    write32(context->base + DART_T8020_TTBR + index * 0x10u + 4u,
            (uint32_t)(address >> 32));
    write32(context->base + DART_T8020_TTBR + index * 0x10u,
            (uint32_t)address | 0x80000000u);
    return 0;
}

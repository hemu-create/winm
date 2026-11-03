#include "../common/apple_registers.h"

#include <stdint.h>
#include <stddef.h>

typedef struct {
    uintptr_t base;
    uint32_t version;
    uint32_t nr_irq;
    uint32_t max_irq;
} aic_context_t;

static uint32_t read32(uintptr_t address) {
    return *(volatile uint32_t *)address;
}

static void write32(uintptr_t address, uint32_t value) {
    *(volatile uint32_t *)address = value;
}

int aic_init(aic_context_t *context, uintptr_t base) {
    if (context == NULL || base == 0) {
        return -1;
    }

    context->base = base;
    context->version = read32(base + AIC_CONFIG) & 0x0000000fu;
    context->nr_irq = read32(base + AIC_INFO) & AIC_INFO_NR_IRQ;
    context->max_irq = 0x400;

    for (uint32_t index = 0; index < (0x400u / 32u); index++) {
        write32(base + AIC_MASK_SET + index * 4u, 0xffffffffu);
    }

    return 0;
}

uint32_t aic_acknowledge(aic_context_t *context) {
    if (context == NULL || context->base == 0) {
        return 0;
    }
    return read32(context->base + AIC_EVENT);
}

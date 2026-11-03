#include <stdint.h>
#include <stddef.h>
#include "../common/apple_registers.h"

typedef struct {
    uintptr_t ecam_base;
    uint32_t segment;
    uint32_t bus_count;
} pcie_context_t;

int pcie_init(pcie_context_t *context, uintptr_t ecam_base, uint32_t segment) {
    if (context == NULL || ecam_base == 0) {
        return -1;
    }

    context->ecam_base = ecam_base;
    context->segment = segment;
    context->bus_count = 256;
    return 0;
}

uint32_t pcie_link_status(pcie_context_t *context) {
    if (context == NULL || context->ecam_base == 0) {
        return 0;
    }
    return *(volatile uint32_t *)(context->ecam_base + PCIE_PORT_LINKSTS);
}

uint32_t pcie_read_config(pcie_context_t *context, uint32_t bus, uint32_t device,
                          uint32_t function, uint32_t offset) {
    if (context == NULL || context->ecam_base == 0) {
        return 0xffffffffu;
    }

    uintptr_t address = context->ecam_base +
                        ((bus & 0xffu) << 20) +
                        ((device & 0x1fu) << 15) +
                        ((function & 0x07u) << 12) +
                        (offset & 0xffcu);
    return *(volatile uint32_t *)address;
}

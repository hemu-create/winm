#include <stdint.h>
#include <stddef.h>
#include "../common/apple_registers.h"

typedef struct {
    uintptr_t dwc3_base;
    uintptr_t phy_base;
    uint32_t role;
} usb_context_t;

int usb_init(usb_context_t *context, uintptr_t dwc3_base, uintptr_t phy_base) {
    if (context == NULL || dwc3_base == 0) {
        return -1;
    }

    context->dwc3_base = dwc3_base;
    context->phy_base = phy_base;
    context->role = 0;
    return 0;
}

int usb_set_role(usb_context_t *context, uint32_t role) {
    if (context == NULL) {
        return -1;
    }

    context->role = role;
    return 0;
}

uint32_t usb_core_id(usb_context_t *context) {
    if (context == NULL || context->dwc3_base == 0) {
        return 0;
    }
    return *(volatile uint32_t *)(context->dwc3_base + DWC3_GSNPSID);
}

uint32_t usb_status(usb_context_t *context) {
    if (context == NULL || context->dwc3_base == 0) {
        return 0;
    }
    return *(volatile uint32_t *)(context->dwc3_base + DWC3_GSTS);
}

#pragma once

#include <stdint.h>

typedef struct _WINM_AIC_OPS {
    uint32_t version;
    uint32_t (*acknowledge)(void *context);
    void (*complete)(void *context, uint32_t irq);
    void (*enable)(void *context, uint32_t irq);
    void (*disable)(void *context, uint32_t irq);
    void (*set_affinity)(void *context, uint32_t irq, uint64_t mpidr);
    uint32_t (*get_priority)(void *context, uint32_t irq);
    void (*set_priority)(void *context, uint32_t irq, uint32_t priority);
} WINM_AIC_OPS;

typedef struct _WINM_AIC_CONTEXT {
    uintptr_t aic_base;
    uint32_t aic_version;
    uint32_t max_irq;
    uint32_t iack_offset;
    uint32_t cap0_offset;
    uint32_t maxnumirq_offset;
    const WINM_AIC_OPS *ops;
} WINM_AIC_CONTEXT;

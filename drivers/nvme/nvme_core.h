#pragma once

#include <stddef.h>
#include <stdint.h>

#include "../common/winm_dma.h"

#define NVME_REG_CAP 0x00u
#define NVME_REG_VS 0x08u
#define NVME_REG_INTMS 0x0cu
#define NVME_REG_INTMC 0x10u
#define NVME_REG_CC 0x14u
#define NVME_REG_CSTS 0x1cu
#define NVME_REG_AQA 0x24u
#define NVME_REG_ASQ 0x28u
#define NVME_REG_ACQ 0x30u
#define NVME_REG_DBS 0x1000u

#define NVME_CC_EN 0x00000001u
#define NVME_CC_IOSQES (6u << 16)
#define NVME_CC_IOCQES (4u << 20)
#define NVME_CC_SHN (2u << 14)

#define NVME_CSTS_RDY 0x00000001u
#define NVME_CSTS_CFS 0x00000002u
#define NVME_CSTS_SHST (3u << 2)

typedef struct _NVME_MMIO {
    void *context;
    uint32_t (*read32)(void *context, uint32_t offset);
    void (*write32)(void *context, uint32_t offset, uint32_t value);
} NVME_MMIO;

typedef struct _NVME_CONTROLLER {
    NVME_MMIO mmio;
    uint64_t cap;
    uint32_t vs;
    uint32_t doorbell_stride;
} NVME_CONTROLLER;

int nvme_core_probe(NVME_CONTROLLER *controller, const NVME_MMIO *mmio);
int nvme_core_wait_ready(NVME_CONTROLLER *controller, int ready);
int nvme_core_reset(NVME_CONTROLLER *controller);
int nvme_core_enable(NVME_CONTROLLER *controller, uint64_t asq, uint64_t acq,
                     uint32_t sq_depth, uint32_t cq_depth);
int nvme_core_shutdown(NVME_CONTROLLER *controller);

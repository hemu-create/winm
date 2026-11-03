#include "nvme_core.h"

#define NVME_WAIT_ITERATIONS 1000000u

static int nvme_valid_depth(uint32_t depth) {
    return depth >= 2 && depth <= 4096 && (depth & (depth - 1)) == 0;
}

static void nvme_write64(NVME_CONTROLLER *controller, uint32_t offset,
                         uint64_t value) {
    controller->mmio.write32(controller->mmio.context, offset,
                             (uint32_t)value);
    controller->mmio.write32(controller->mmio.context, offset + 4,
                             (uint32_t)(value >> 32));
}

int nvme_core_probe(NVME_CONTROLLER *controller, const NVME_MMIO *mmio) {
    uint64_t lo;
    uint64_t hi;

    if (controller == NULL || mmio == NULL || mmio->read32 == NULL ||
        mmio->write32 == NULL) {
        return -1;
    }

    controller->mmio = *mmio;
    lo = mmio->read32(mmio->context, NVME_REG_CAP);
    hi = mmio->read32(mmio->context, NVME_REG_CAP + 4);
    controller->cap = (hi << 32) | lo;
    controller->vs = mmio->read32(mmio->context, NVME_REG_VS);
    controller->doorbell_stride = (uint32_t)((controller->cap >> 32) & 0xfu);
    return 0;
}

int nvme_core_wait_ready(NVME_CONTROLLER *controller, int ready) {
    uint32_t index;
    uint32_t status;

    if (controller == NULL || controller->mmio.read32 == NULL) {
        return -1;
    }

    for (index = 0; index < NVME_WAIT_ITERATIONS; index++) {
        status = controller->mmio.read32(controller->mmio.context,
                                         NVME_REG_CSTS);
        if ((status & NVME_CSTS_CFS) != 0) {
            return -1;
        }
        if (((status & NVME_CSTS_RDY) != 0) == (ready != 0)) {
            return 0;
        }
    }

    return -1;
}

int nvme_core_reset(NVME_CONTROLLER *controller) {
    if (controller == NULL || controller->mmio.write32 == NULL) {
        return -1;
    }

    controller->mmio.write32(controller->mmio.context, NVME_REG_CC, 0);
    return nvme_core_wait_ready(controller, 0);
}

int nvme_core_enable(NVME_CONTROLLER *controller, uint64_t asq, uint64_t acq,
                     uint32_t sq_depth, uint32_t cq_depth) {
    uint32_t aqa;
    uint32_t cc;

    if (controller == NULL || controller->mmio.write32 == NULL ||
        asq == 0 || acq == 0 || !nvme_valid_depth(sq_depth) ||
        !nvme_valid_depth(cq_depth)) {
        return -1;
    }

    if (nvme_core_reset(controller) != 0) {
        return -1;
    }

    aqa = ((cq_depth - 1) << 16) | (sq_depth - 1);
    controller->mmio.write32(controller->mmio.context, NVME_REG_AQA, aqa);
    nvme_write64(controller, NVME_REG_ASQ, asq);
    nvme_write64(controller, NVME_REG_ACQ, acq);

    cc = NVME_CC_EN | NVME_CC_IOSQES | NVME_CC_IOCQES;
    controller->mmio.write32(controller->mmio.context, NVME_REG_CC, cc);
    return nvme_core_wait_ready(controller, 1);
}

int nvme_core_shutdown(NVME_CONTROLLER *controller) {
    uint32_t index;
    uint32_t status;
    uint32_t cc;

    if (controller == NULL || controller->mmio.write32 == NULL) {
        return -1;
    }

    cc = controller->mmio.read32(controller->mmio.context, NVME_REG_CC);
    controller->mmio.write32(controller->mmio.context, NVME_REG_CC,
                             cc | NVME_CC_SHN);

    for (index = 0; index < NVME_WAIT_ITERATIONS; index++) {
        status = controller->mmio.read32(controller->mmio.context,
                                         NVME_REG_CSTS);
        if ((status & NVME_CSTS_SHST) == NVME_CSTS_SHST) {
            return 0;
        }
    }

    return -1;
}

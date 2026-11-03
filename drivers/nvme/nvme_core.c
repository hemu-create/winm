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

static uint32_t nvme_doorbell_stride_bytes(const NVME_CONTROLLER *controller) {
    uint32_t raw = controller->doorbell_stride;
    if (raw > 15u) {
        raw = 0;
    }
    return 1u << (2u + raw);
}

int nvme_core_admin_queue_init(NVME_CONTROLLER *controller,
                               NVME_ADMIN_QUEUE *queue, volatile uint32_t *sq,
                               volatile uint32_t *cq, uint32_t sq_depth,
                               uint32_t cq_depth) {
    uint32_t index;

    if (controller == NULL || queue == NULL || sq == NULL || cq == NULL ||
        !nvme_valid_depth(sq_depth) || !nvme_valid_depth(cq_depth)) {
        return -1;
    }

    queue->sq = sq;
    queue->cq = cq;
    queue->sq_depth = sq_depth;
    queue->cq_depth = cq_depth;
    queue->sq_tail = 0;
    queue->cq_head = 0;
    queue->next_command_id = 1;
    queue->phase = 1;

    for (index = 0; index < sq_depth * NVME_ADMIN_SQ_ENTRY_DWORDS; index++) {
        sq[index] = 0;
    }
    for (index = 0; index < cq_depth * NVME_ADMIN_CQ_ENTRY_DWORDS; index++) {
        cq[index] = 0;
    }

    return 0;
}

int nvme_core_admin_submit(NVME_CONTROLLER *controller,
                           NVME_ADMIN_QUEUE *queue, uint8_t opcode,
                           uint32_t nsid, uint64_t prp1, uint32_t cdw10,
                           uint32_t cdw11, uint16_t *command_id) {
    uint32_t *entry;
    uint16_t cid;
    uint32_t tail;

    if (controller == NULL || queue == NULL || queue->sq == NULL ||
        controller->mmio.write32 == NULL) {
        return -1;
    }

    cid = queue->next_command_id;
    if (cid == 0) {
        cid = 1;
    }
    queue->next_command_id = (uint16_t)(cid + 1);
    if (queue->next_command_id == 0) {
        queue->next_command_id = 1;
    }

    tail = queue->sq_tail;
    entry = (uint32_t *)(queue->sq + (tail * NVME_ADMIN_SQ_ENTRY_DWORDS));

    for (tail = 0; tail < NVME_ADMIN_SQ_ENTRY_DWORDS; tail++) {
        entry[tail] = 0;
    }

    entry[0] = (uint32_t)opcode | ((uint32_t)cid << 16);
    entry[1] = nsid;
    entry[6] = (uint32_t)prp1;
    entry[7] = (uint32_t)(prp1 >> 32);
    entry[10] = cdw10;
    entry[11] = cdw11;

    queue->sq_tail = (uint16_t)((queue->sq_tail + 1) % queue->sq_depth);
    controller->mmio.write32(controller->mmio.context, NVME_REG_DBS,
                             queue->sq_tail);

    if (command_id != NULL) {
        *command_id = cid;
    }

    return 0;
}

int nvme_core_admin_poll(NVME_CONTROLLER *controller,
                         NVME_ADMIN_QUEUE *queue, uint16_t *command_id,
                         uint32_t *status_code) {
    uint32_t *entry;
    uint32_t status;
    uint32_t stride;

    if (controller == NULL || queue == NULL || queue->cq == NULL ||
        controller->mmio.write32 == NULL) {
        return -1;
    }

    entry = (uint32_t *)(queue->cq + (queue->cq_head * NVME_ADMIN_CQ_ENTRY_DWORDS));
    status = entry[3];
    if (((status >> 16) & 0x1u) != queue->phase) {
        return 0;
    }

    if (command_id != NULL) {
        *command_id = (uint16_t)(status & 0xffffu);
    }
    if (status_code != NULL) {
        *status_code = (status >> 17) & 0x7ffu;
    }

    queue->cq_head = (uint16_t)((queue->cq_head + 1) % queue->cq_depth);
    if (queue->cq_head == 0) {
        queue->phase ^= 1;
    }

    stride = nvme_doorbell_stride_bytes(controller);
    controller->mmio.write32(controller->mmio.context,
                             NVME_REG_DBS + stride,
                             queue->cq_head);

    return 1;
}

int nvme_core_identify_controller(NVME_CONTROLLER *controller,
                                 NVME_ADMIN_QUEUE *queue, uint64_t prp1,
                                 uint16_t *command_id) {
    return nvme_core_admin_submit(controller, queue, 0x06, 0, prp1, 1, 0,
                                 command_id);
}

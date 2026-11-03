#include "nvme.h"

#include <stddef.h>

uint64_t nvme_alloc_dma_buffer(uint32_t size) {
    /*
     * Phase 2 placeholder. Real implementation must allocate from a 16 KiB
     * aligned pool and, if the requested size is not a multiple of the DART
     * granule, round up or use a bounce buffer.
     */
    (void)size;
    return 0;
}

void nvme_free_dma_buffer(uint64_t address, uint32_t size) {
    (void)address;
    (void)size;
}

int nvme_map_prp(NVME_PRP *prp, uint64_t buffer, uint32_t size) {
    if (prp == NULL || buffer == 0 || size == 0) {
        return -1;
    }

    if ((buffer & WINM_CPU_PAGE_MASK) != 0) {
        return -1;
    }

    if (size <= NVME_PAGE_SIZE) {
        prp->pages[0] = buffer;
        prp->pages[1] = 0;
        prp->count = 1;
        return 0;
    }

    if (size <= (2 * NVME_PAGE_SIZE)) {
        prp->pages[0] = buffer;
        prp->pages[1] = buffer + NVME_PAGE_SIZE;
        prp->count = 2;
        return 0;
    }

    return -1;
}

int nvme_submit_admin_command(NVME_QUEUE_PAIR *qp, uint32_t opcode,
                             uint64_t prp1, uint64_t prp2, uint32_t nsid) {
    (void)qp;
    (void)opcode;
    (void)prp1;
    (void)prp2;
    (void)nsid;
    return -1;
}

int nvme_poll_completion(NVME_QUEUE_PAIR *qp, uint32_t *status) {
    (void)qp;
    if (status != NULL) {
        *status = 0;
    }
    return -1;
}

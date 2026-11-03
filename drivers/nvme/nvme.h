#pragma once

#include <stdint.h>

#include "../common/winm_dma.h"

#define NVME_PAGE_SIZE WINM_CPU_PAGE_SIZE
#define DART_GRANULE WINM_DART_GRANULE
#define DART_GRANULE_MASK WINM_DART_GRANULE_MASK

typedef struct _NVME_QUEUE_PAIR {
    uint64_t sq_base;
    uint32_t sq_size;
    uint32_t sq_tail;
    uint64_t cq_base;
    uint32_t cq_size;
    uint32_t cq_head;
    uint32_t phase;
} NVME_QUEUE_PAIR;

typedef struct _NVME_PRP {
    uint64_t pages[2];
    uint32_t count;
} NVME_PRP;

static inline int dart_aligned(uint64_t address, uint32_t size) {
    return (address & DART_GRANULE_MASK) == 0 &&
           (size & DART_GRANULE_MASK) == 0;
}

uint64_t nvme_alloc_dma_buffer(uint32_t size);
void nvme_free_dma_buffer(uint64_t address, uint32_t size);
int nvme_map_prp(NVME_PRP *prp, uint64_t buffer, uint32_t size);
int nvme_submit_admin_command(NVME_QUEUE_PAIR *qp, uint32_t opcode,
                             uint64_t prp1, uint64_t prp2, uint32_t nsid);
int nvme_poll_completion(NVME_QUEUE_PAIR *qp, uint32_t *status);

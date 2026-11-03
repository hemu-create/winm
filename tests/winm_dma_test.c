#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include "../drivers/common/winm_dma.h"
#include "../drivers/nvme/nvme.h"

static int failures;

static void check(int condition, const char *message) {
    if (!condition) {
        printf("FAIL: %s\n", message);
        failures++;
    }
}

static void *test_alloc(size_t size, void *context) {
    (void)context;
    return malloc(size);
}

static void test_free(void *address, void *context) {
    (void)context;
    free(address);
}

int main(void) {
    WINM_DMA_BUFFER buffer;
    NVME_PRP prp;

    check(WINM_DART_GRANULE == 16384, "DART granule is 16 KiB");
    check(winm_dma_round_up(1) == 16384, "small size rounds to one granule");
    check(winm_dma_round_up(16384) == 16384, "aligned size stays aligned");
    check(winm_dma_round_up(16385) == 32768, "unaligned size rounds up");
    check(winm_dma_round_up((size_t)-1) == 0, "overflow size rejected");

    check(winm_dma_needs_bounce(0x4000, 16384) == 0, "aligned transfer");
    check(winm_dma_needs_bounce(0x4000, 4096) == 1, "4 KiB transfer needs bounce");
    check(winm_dma_needs_bounce(0x4001, 16384) == 1, "misaligned address needs bounce");

    check(winm_dma_alloc(&buffer, 100, test_alloc, test_free, NULL) == 0, "allocate");
    check(buffer.size == 16384, "allocated size is rounded");
    check(((uintptr_t)buffer.aligned & WINM_DART_GRANULE_MASK) == 0, "aligned address");
    winm_dma_free(&buffer);
    check(buffer.raw == NULL && buffer.aligned == NULL && buffer.size == 0, "free clears buffer");

    check(nvme_map_prp(&prp, 0x10000, 4096) == 0, "single page PRP");
    check(prp.count == 1 && prp.pages[0] == 0x10000, "single PRP contents");
    check(nvme_map_prp(&prp, 0x10000, 8192) == 0, "two page PRP");
    check(prp.count == 2 && prp.pages[1] == 0x11000, "two page PRP contents");
    check(nvme_map_prp(&prp, 0x10000, 12288) == -1, "large PRP rejected");
    check(nvme_map_prp(&prp, 0x10001, 4096) == -1, "unaligned PRP rejected");

    if (failures != 0) {
        printf("%d failure(s)\n", failures);
        return 1;
    }

    printf("winm_dma_test: ok\n");
    return 0;
}

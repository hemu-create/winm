#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "../drivers/nvme/nvme_core.h"

typedef struct _FAKE_MMIO {
    uint8_t registers[0x2000];
    int ready_on_enable;
    int complete_shutdown;
} FAKE_MMIO;

static int failures;

static void check(int condition, const char *message) {
    if (!condition) {
        printf("FAIL: %s\n", message);
        failures++;
    }
}

static uint32_t fake_read32(void *context, uint32_t offset) {
    FAKE_MMIO *fake = context;
    uint32_t value = 0;
    if (offset + sizeof(value) <= sizeof(fake->registers)) {
        memcpy(&value, fake->registers + offset, sizeof(value));
    }
    return value;
}

static void fake_write32(void *context, uint32_t offset, uint32_t value) {
    FAKE_MMIO *fake = context;
    if (offset + sizeof(value) > sizeof(fake->registers)) {
        return;
    }
    memcpy(fake->registers + offset, &value, sizeof(value));

    if (offset == NVME_REG_CC) {
        uint32_t status = 0;
        if ((value & NVME_CC_EN) && fake->ready_on_enable) {
            status |= NVME_CSTS_RDY;
        }
        if ((value & NVME_CC_SHN) && fake->complete_shutdown) {
            status |= NVME_CSTS_SHST;
        }
        memcpy(fake->registers + NVME_REG_CSTS, &status, sizeof(status));
    }
}

int main(void) {
    FAKE_MMIO fake;
    NVME_MMIO mmio;
    NVME_CONTROLLER controller;
    uint32_t value;

    memset(&fake, 0, sizeof(fake));
    fake.ready_on_enable = 1;
    fake.complete_shutdown = 1;

    value = 0x00010400;
    memcpy(fake.registers + NVME_REG_CAP, &value, sizeof(value));
    value = 0;
    memcpy(fake.registers + NVME_REG_CAP + 4, &value, sizeof(value));
    value = 0x00010400;
    memcpy(fake.registers + NVME_REG_VS, &value, sizeof(value));

    mmio.context = &fake;
    mmio.read32 = fake_read32;
    mmio.write32 = fake_write32;

    check(nvme_core_probe(&controller, &mmio) == 0, "probe");
    check(controller.vs == 0x00010400, "version");
    check(controller.doorbell_stride == 0, "doorbell stride");

    check(nvme_core_enable(&controller, 0x1000, 0x2000, 2, 2) == 0, "enable");
    check(fake_read32(&fake, NVME_REG_AQA) == 0x00010001u, "AQA");
    check(fake_read32(&fake, NVME_REG_ASQ) == 0x1000, "ASQ low");
    check(fake_read32(&fake, NVME_REG_ASQ + 4) == 0, "ASQ high");
    check(fake_read32(&fake, NVME_REG_ACQ) == 0x2000, "ACQ low");
    check(fake_read32(&fake, NVME_REG_CC) == 0x00460001u, "CC");
    check((fake_read32(&fake, NVME_REG_CSTS) & NVME_CSTS_RDY) != 0, "ready");

    check(nvme_core_reset(&controller) == 0, "reset");
    check((fake_read32(&fake, NVME_REG_CSTS) & NVME_CSTS_RDY) == 0, "not ready");

    check(nvme_core_enable(&controller, 0x1000, 0x2000, 4, 4) == 0, "enable 4");
    check(fake_read32(&fake, NVME_REG_AQA) == 0x00030003u, "AQA 4");
    check(nvme_core_shutdown(&controller) == 0, "shutdown");
    check((fake_read32(&fake, NVME_REG_CSTS) & NVME_CSTS_SHST) == NVME_CSTS_SHST, "shutdown complete");

    check(nvme_core_enable(&controller, 0x1000, 0x2000, 3, 4) == -1, "non-power-of-two depth");
    check(nvme_core_enable(&controller, 0, 0x2000, 2, 2) == -1, "zero ASQ");
    check(nvme_core_probe(&controller, NULL) == -1, "null mmio");

    memcpy(fake.registers + NVME_REG_CSTS, &(uint32_t){NVME_CSTS_CFS}, sizeof(uint32_t));
    check(nvme_core_wait_ready(&controller, 1) == -1, "controller fatal status");

    {
        volatile uint32_t sq[2 * NVME_ADMIN_SQ_ENTRY_DWORDS];
        volatile uint32_t cq[2 * NVME_ADMIN_CQ_ENTRY_DWORDS];
        NVME_ADMIN_QUEUE queue;
        uint16_t cid = 0;
        uint32_t status = 0;
        int complete;

        check(nvme_core_admin_queue_init(&controller, &queue, sq, cq, 2, 2) == 0,
              "admin queue init");
        check(queue.phase == 1, "admin queue phase");

        check(nvme_core_identify_controller(&controller, &queue, 0x4000, &cid) == 0,
              "identify submit");
        check(cid == 1, "identify command id");
        check(sq[0] == (0x06u | (1u << 16)), "identify opcode/cid");
        check(sq[6] == 0x4000u, "identify PRP low");
        check(sq[10] == 1u, "identify CNS");
        check(fake_read32(&fake, NVME_REG_DBS) == 1u, "SQ doorbell");

        cq[3] = 2u | (1u << 16);
        complete = nvme_core_admin_poll(&controller, &queue, &cid, &status);
        check(complete == 1, "admin poll complete");
        check(cid == 2, "completion cid");
        check(status == 0, "completion status");
        check(fake_read32(&fake, NVME_REG_DBS + 4) == 1u, "CQ doorbell");
        check(queue.phase == 1, "phase unchanged");

        cq[16 + 3] = 3u | (1u << 16);
        complete = nvme_core_admin_poll(&controller, &queue, &cid, &status);
        check(complete == 1, "admin poll wrap complete");
        check(cid == 3, "wrapped completion cid");
        check(queue.cq_head == 0, "queue head wrapped");
        check(queue.phase == 0, "phase toggled on wrap");

        cq[3] = 4u | (1u << 16);
        complete = nvme_core_admin_poll(&controller, &queue, &cid, &status);
        check(complete == 0, "old phase not complete");

        check(nvme_core_admin_queue_init(NULL, &queue, sq, cq, 2, 2) == -1,
              "null controller queue init");
    }

    if (failures != 0) {
        printf("%d failure(s)\n", failures);
        return 1;
    }

    printf("nvme_core_test: ok\n");
    return 0;
}

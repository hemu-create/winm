#include <stdint.h>
#include <stddef.h>

typedef struct {
    uintptr_t pmu_base;
    uint32_t thermal_limit;
    uint32_t fan_duty;
} pmu_context_t;

int pmu_init(pmu_context_t *context, uintptr_t pmu_base) {
    if (context == NULL || pmu_base == 0) {
        return -1;
    }

    context->pmu_base = pmu_base;
    context->thermal_limit = 85;
    context->fan_duty = 30;
    return 0;
}

int pmu_read_temperature(pmu_context_t *context) {
    if (context == NULL || context->pmu_base == 0) {
        return -1;
    }

    return 40;
}

int pmu_set_fan_duty(pmu_context_t *context, uint32_t duty) {
    if (context == NULL || duty > 100) {
        return -1;
    }

    context->fan_duty = duty;
    return 0;
}

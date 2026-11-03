#pragma once

#ifdef _KERNEL_MODE
#include <ntddk.h>
#else
#include <windows.h>
#include <winioctl.h>
#endif

#define IOCTL_WINM_NVME_GET_INFO \
    CTL_CODE(FILE_DEVICE_UNKNOWN, 0x800, METHOD_BUFFERED, FILE_READ_ACCESS)

typedef struct _WINM_NVME_INFO {
    UINT64 Cap;
    UINT32 Version;
    UINT32 DoorbellStride;
    UINT64 Bar0Length;
} WINM_NVME_INFO, *PWINM_NVME_INFO;

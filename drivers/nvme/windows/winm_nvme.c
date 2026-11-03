#include <ntddk.h>
#include <wdf.h>

#include "nvme_core.h"
#include "winm_nvme_ioctl.h"

typedef struct _DEVICE_CONTEXT {
    NVME_CONTROLLER Controller;
    PVOID Bar0;
    SIZE_T Bar0Length;
} DEVICE_CONTEXT, *PDEVICE_CONTEXT;

WDF_DECLARE_CONTEXT_TYPE_WITH_NAME(DEVICE_CONTEXT, WinMNvmeGetContext)

static UINT32 WinMNvmeRead32(void *Context, UINT32 Offset) {
    PDEVICE_CONTEXT DeviceContext = (PDEVICE_CONTEXT)Context;
    return (UINT32)READ_REGISTER_ULONG(
        (PULONG)((PUCHAR)DeviceContext->Bar0 + Offset));
}

static void WinMNvmeWrite32(void *Context, UINT32 Offset, UINT32 Value) {
    PDEVICE_CONTEXT DeviceContext = (PDEVICE_CONTEXT)Context;
    WRITE_REGISTER_ULONG((PULONG)((PUCHAR)DeviceContext->Bar0 + Offset),
                         (ULONG)Value);
}

static NTSTATUS WinMNvmeEvtDevicePrepareHardware(
    WDFDEVICE Device,
    WDFCMRESLIST ResourcesRaw,
    WDFCMRESLIST ResourcesTranslated) {
    PDEVICE_CONTEXT DeviceContext = WinMNvmeGetContext(Device);
    ULONG Index;
    ULONG Count;
    NVME_MMIO Mmio;

    UNREFERENCED_PARAMETER(ResourcesTranslated);

    Count = WdfCmResourceListGetCount(ResourcesRaw);
    for (Index = 0; Index < Count; Index++) {
        PCM_PARTIAL_RESOURCE_DESCRIPTOR Descriptor =
            WdfCmResourceListGetDescriptor(ResourcesRaw, Index);
        if (Descriptor != NULL && Descriptor->Type == CmResourceTypeMemory) {
            DeviceContext->Bar0Length = Descriptor->u.Memory.Length;
            DeviceContext->Bar0 = MmMapIoSpace(
                Descriptor->u.Memory.Start,
                DeviceContext->Bar0Length,
                MmNonCached);
            break;
        }
    }

    if (DeviceContext->Bar0 == NULL) {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    Mmio.context = DeviceContext;
    Mmio.read32 = WinMNvmeRead32;
    Mmio.write32 = WinMNvmeWrite32;

    if (nvme_core_probe(&DeviceContext->Controller, &Mmio) != 0) {
        MmUnmapIoSpace(DeviceContext->Bar0,
                        DeviceContext->Bar0Length);
        DeviceContext->Bar0 = NULL;
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    KdPrintEx((DPFLTR_IHVDRIVER_ID, DPFLTR_INFO_LEVEL,
               "WinMNvme: CAP=%I64x VS=%lx DSTRD=%lu\n",
               DeviceContext->Controller.cap,
               DeviceContext->Controller.vs,
               DeviceContext->Controller.doorbell_stride));

    return STATUS_SUCCESS;
}

static NTSTATUS WinMNvmeEvtDeviceReleaseHardware(
    WDFDEVICE Device,
    WDFCMRESLIST ResourcesTranslated) {
    PDEVICE_CONTEXT DeviceContext = WinMNvmeGetContext(Device);

    UNREFERENCED_PARAMETER(ResourcesTranslated);

    if (DeviceContext->Bar0 != NULL) {
        MmUnmapIoSpace(DeviceContext->Bar0,
                        DeviceContext->Bar0Length);
        DeviceContext->Bar0 = NULL;
        DeviceContext->Bar0Length = 0;
    }

    return STATUS_SUCCESS;
}

static void WinMNvmeEvtIoDeviceControl(
    WDFQUEUE Queue,
    WDFREQUEST Request,
    size_t OutputBufferLength,
    size_t InputBufferLength,
    ULONG IoControlCode) {
    PDEVICE_CONTEXT DeviceContext = WinMNvmeGetContext(WdfIoQueueGetDevice(Queue));
    PWINM_NVME_INFO Info = NULL;
    NTSTATUS Status;

    UNREFERENCED_PARAMETER(InputBufferLength);

    if (IoControlCode != IOCTL_WINM_NVME_GET_INFO) {
        WdfRequestComplete(Request, STATUS_INVALID_DEVICE_REQUEST);
        return;
    }

    if (OutputBufferLength < sizeof(WINM_NVME_INFO)) {
        WdfRequestComplete(Request, STATUS_BUFFER_TOO_SMALL);
        return;
    }

    Status = WdfRequestRetrieveOutputBuffer(
        Request, sizeof(WINM_NVME_INFO), (PVOID *)&Info, NULL);
    if (!NT_SUCCESS(Status)) {
        WdfRequestComplete(Request, Status);
        return;
    }

    Info->Cap = DeviceContext->Controller.cap;
    Info->Version = DeviceContext->Controller.vs;
    Info->DoorbellStride = DeviceContext->Controller.doorbell_stride;
    Info->Bar0Length = DeviceContext->Bar0Length;

    WdfRequestCompleteWithInformation(Request, STATUS_SUCCESS,
                                      sizeof(WINM_NVME_INFO));
}

static NTSTATUS WinMNvmeEvtDeviceAdd(
    WDFDRIVER Driver,
    PWDFDEVICE_INIT DeviceInit) {
    WDF_OBJECT_ATTRIBUTES Attributes;
    WDF_PNPPOWER_EVENT_CALLBACKS PnpPowerCallbacks;
    WDF_IO_QUEUE_CONFIG QueueConfig;
    WDFDEVICE Device;
    PDEVICE_CONTEXT DeviceContext;
    NTSTATUS Status;

    UNREFERENCED_PARAMETER(Driver);

    WDF_PNPPOWER_EVENT_CALLBACKS_INIT(&PnpPowerCallbacks);
    PnpPowerCallbacks.EvtDevicePrepareHardware =
        WinMNvmeEvtDevicePrepareHardware;
    PnpPowerCallbacks.EvtDeviceReleaseHardware =
        WinMNvmeEvtDeviceReleaseHardware;
    WdfDeviceInitSetPnpPowerEventCallbacks(DeviceInit, &PnpPowerCallbacks);

    WdfDeviceInitSetIoType(DeviceInit, WdfDeviceIoBuffered);

    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&Attributes, DEVICE_CONTEXT);
    Status = WdfDeviceCreate(&DeviceInit, &Attributes, &Device);
    if (!NT_SUCCESS(Status)) {
        return Status;
    }

    DeviceContext = WinMNvmeGetContext(Device);
    DeviceContext->Bar0 = NULL;
    DeviceContext->Bar0Length = 0;
    RtlZeroMemory(&DeviceContext->Controller,
                  sizeof(DeviceContext->Controller));

    WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(
        &QueueConfig, WdfIoQueueDispatchSequential);
    QueueConfig.EvtIoDeviceControl = WinMNvmeEvtIoDeviceControl;

    Status = WdfIoQueueCreate(Device, &QueueConfig, WDF_NO_OBJECT_ATTRIBUTES,
                              WDF_NO_HANDLE);
    return Status;
}

NTSTATUS DriverEntry(
    PDRIVER_OBJECT DriverObject,
    PUNICODE_STRING RegistryPath) {
    WDF_DRIVER_CONFIG Config;

    WDF_DRIVER_CONFIG_INIT(&Config, WinMNvmeEvtDeviceAdd);
    return WdfDriverCreate(DriverObject, RegistryPath,
                           WDF_NO_OBJECT_ATTRIBUTES, &Config, WDF_NO_HANDLE);
}

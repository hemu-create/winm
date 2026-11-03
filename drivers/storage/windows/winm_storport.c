#include <ntddk.h>
#include <storport.h>

typedef struct _DEVICE_EXTENSION {
    ULONG Reserved;
} DEVICE_EXTENSION, *PDEVICE_EXTENSION;

static ULONG HwFindAdapter(
    PVOID DeviceExtension,
    PVOID HwContext,
    PVOID BusInformation,
    PCHAR ArgumentString,
    PPORT_CONFIGURATION_INFORMATION ConfigInfo,
    PBOOLEAN Again) {
    UNREFERENCED_PARAMETER(DeviceExtension);
    UNREFERENCED_PARAMETER(HwContext);
    UNREFERENCED_PARAMETER(BusInformation);
    UNREFERENCED_PARAMETER(ArgumentString);

    ConfigInfo->MaximumTransferLength = 0x20000;
    ConfigInfo->NumberOfPhysicalBreaks = 128;
    ConfigInfo->AlignmentMask = 0x3;
    ConfigInfo->DeviceExtensionSize = sizeof(DEVICE_EXTENSION);
    *Again = FALSE;
    return SP_RETURN_FOUND;
}

static BOOLEAN HwInitialize(PVOID DeviceExtension) {
    UNREFERENCED_PARAMETER(DeviceExtension);
    return TRUE;
}

static BOOLEAN HwStartIo(PVOID DeviceExtension, PSCSI_REQUEST_BLOCK Srb) {
    UNREFERENCED_PARAMETER(DeviceExtension);
    UNREFERENCED_PARAMETER(Srb);
    return TRUE;
}

static BOOLEAN HwInterrupt(PVOID DeviceExtension) {
    UNREFERENCED_PARAMETER(DeviceExtension);
    return TRUE;
}

NTSTATUS DriverEntry(PDRIVER_OBJECT DriverObject,
                    PUNICODE_STRING RegistryPath) {
    HW_INITIALIZATION_DATA InitData;

    RtlZeroMemory(&InitData, sizeof(InitData));
    InitData.HwInitializationDataSize = sizeof(HW_INITIALIZATION_DATA);
    InitData.HwFindAdapter = HwFindAdapter;
    InitData.HwInitialize = HwInitialize;
    InitData.HwStartIo = HwStartIo;
    InitData.HwInterrupt = HwInterrupt;
    InitData.DeviceExtensionSize = sizeof(DEVICE_EXTENSION);
    InitData.SrbExtensionSize = 0;
    InitData.NumberOfAccessRanges = 1;
    InitData.MapBuffers = STOR_MAP_NON_READ_WRITE_BUFFERS;
    InitData.NeedPhysicalAddresses = TRUE;
    InitData.TaggedQueuing = TRUE;
    InitData.AutoRequestSense = TRUE;
    InitData.MultipleRequestPerLu = TRUE;

    return StorPortInitialize(DriverObject, RegistryPath, &InitData, NULL);
}

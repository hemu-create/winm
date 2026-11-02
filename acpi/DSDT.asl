DefinitionBlock ("DSDT.aml", "DSDT", 2, "WINM", "WINM", 0x00000001)
{
    Scope (_SB)
    {
        Device (CPU0)
        {
            Name (_HID, "ACPI0007")
            Name (_UID, 0)
        }

        Device (AIC0)
        {
            Name (_HID, "APLC0001")
            Name (_UID, 0)
        }

        Device (URT0)
        {
            Name (_HID, "APLC0002")
            Name (_UID, 0)
        }
    }
}

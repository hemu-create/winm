OUTPUT_FORMAT("elf64-littleaarch64")
OUTPUT_ARCH(aarch64)
ENTRY(_start)

SECTIONS
{
    . = 0;
    __image_start = .;

    .text : ALIGN(4096)
    {
        KEEP(*(.text.boot))
        KEEP(*(.text.vectors))
        KEEP(*(.text.el1))
        *(.text .text.*)
    }

    .rodata : ALIGN(4096)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4096)
    {
        *(.data .data.*)
        *(.got .got.*)
    }

    .bss (NOLOAD) : ALIGN(4096)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
        . += 0x10000;
        __stack_top = .;
        __bss_end = .;
    }

    . = ALIGN(4096);
    __image_end = .;
    __image_size = __image_end - __image_start;

    /DISCARD/ :
    {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame .eh_frame.*)
    }
}

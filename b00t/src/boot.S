.section .text.boot, "ax"
.global _start
.type _start, %function
_start:
    b       _entry
    nop
    .quad   0
    .quad   __image_size
    .quad   0
    .quad   0
    .quad   0
    .quad   0
    .ascii  "ARM\x64"
    .long   0

_entry:
    mov     x19, x0

    adrp    x1, __stack_top
    add     x1, x1, :lo12:__stack_top
    and     x1, x1, #~15
    mov     sp, x1

    adrp    x2, __bss_start
    add     x2, x2, :lo12:__bss_start
    adrp    x3, __bss_end
    add     x3, x3, :lo12:__bss_end

1:
    cmp     x2, x3
    b.hs    2f
    str     xzr, [x2], #8
    b       1b

2:
    dsb     sy
    isb

    adr     x0, __exception_vectors
    msr     VBAR_EL2, x0
    isb

    mov     x0, x19
    bl      rust_main

3:
    wfe
    b       3b
.size _start, . - _start

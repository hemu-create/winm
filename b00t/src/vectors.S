.section .text.vectors, "ax"
.align 11
.global __exception_vectors
.type __exception_vectors, %function
__exception_vectors:
    .macro vector_entry num
    mov x9, #\num
    b common_exception_handler
    .space 120
    .endm

    vector_entry 0
    vector_entry 1
    vector_entry 2
    vector_entry 3
    vector_entry 4
    vector_entry 5
    vector_entry 6
    vector_entry 7
    vector_entry 8
    vector_entry 9
    vector_entry 10
    vector_entry 11
    vector_entry 12
    vector_entry 13
    vector_entry 14
    vector_entry 15

common_exception_handler:
    sub sp, sp, #256
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]
    str x30, [sp, #240]

    mov x0, x9
    mrs x1, ESR_EL2
    mrs x2, FAR_EL2
    mrs x3, ELR_EL2
    mov x4, sp
    bl rust_exception_handler

    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldr x30, [sp, #240]
    add sp, sp, #256
    eret

.size __exception_vectors, . - __exception_vectors

.section .text.el1, "ax"
.global el1_test
.type el1_test, %function
el1_test:
    ldr x1, =0x2a1200000

    .macro el1_putc ch
    1:
    ldr w2, [x1, #0x10]
    tst w2, #2
    beq 1b
    mov w2, #\ch
    str w2, [x1, #0x20]
    .endm

    el1_putc 'E'
    el1_putc 'L'
    el1_putc '1'
    el1_putc ' '
    el1_putc 'O'
    el1_putc 'K'
    el1_putc '\n'

2:
    wfe
    b 2b
.size el1_test, . - el1_test

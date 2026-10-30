.globl memcpy
memcpy:
    mov x3, x0
    cbz x2, memcpy_end
memcpy_loop:
    ldrb w4, [x1], #1
    strb w4, [x3], #1
    subs x2, x2, #1
    b.ne memcpy_loop
memcpy_end:
    ret
.globl memmove
memmove:
    cmp x0, x1
    b.ls memcpy
    add x3, x1, x2
    cmp x0, x3
    b.hs memcpy
    add x4, x0, x2
memmove_down:
    ldrb w5, [x3, #-1]!
    strb w5, [x4, #-1]!
    subs x2, x2, #1
    b.ne memmove_down
    ret
.globl memset
memset:
    mov x3, x0
    cbz x2, memset_end
memset_loop:
    strb w1, [x3], #1
    subs x2, x2, #1
    b.ne memset_loop
memset_end:
    ret
.globl memcmp
.globl bcmp
bcmp:
memcmp:
    cbz x2, memcmp_equal
memcmp_loop:
    ldrb w3, [x0], #1
    ldrb w4, [x1], #1
    subs w3, w3, w4
    b.ne memcmp_differ
    subs x2, x2, #1
    b.ne memcmp_loop
memcmp_equal:
    mov w0, #0
    ret
memcmp_differ:
    mov w0, w3
    ret
.globl wcslen
wcslen:
    mov x1, x0
wcslen_loop:
    ldrh w2, [x1], #2
    cbnz w2, wcslen_loop
    sub x0, x1, x0
    sub x0, x0, #2
    lsr x0, x0, #1
    ret
.globl __chkstk
__chkstk:
    lsl x16, x15, #4
    mov x17, sp
chkstk_loop:
    sub x17, x17, #4096
    subs x16, x16, #4096
    ldr xzr, [x17]
    b.gt chkstk_loop
    ret
.section .text,"xr",discard,strlen
.globl strlen
strlen:
    mov x1, x0
strlen_loop:
    ldrb w2, [x1], #1
    cbnz w2, strlen_loop
    sub x0, x1, x0
    sub x0, x0, #1
    ret

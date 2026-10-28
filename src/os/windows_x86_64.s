.globl memcpy
memcpy:
    mov rax, rcx
    push rdi
    push rsi
    mov rdi, rcx
    mov rsi, rdx
    mov rcx, r8
    rep movsb
    pop rsi
    pop rdi
    ret
.globl memmove
memmove:
    mov rax, rcx
    cmp rcx, rdx
    jbe memmove_up
    lea r9, [rdx + r8]
    cmp rcx, r9
    jae memmove_up
    push rdi
    push rsi
    lea rdi, [rcx + r8 - 1]
    lea rsi, [rdx + r8 - 1]
    mov rcx, r8
    std
    rep movsb
    cld
    pop rsi
    pop rdi
    ret
memmove_up:
    push rdi
    push rsi
    mov rdi, rcx
    mov rsi, rdx
    mov rcx, r8
    rep movsb
    pop rsi
    pop rdi
    ret
.globl memset
memset:
    mov r9, rcx
    push rdi
    mov rdi, rcx
    mov eax, edx
    mov rcx, r8
    rep stosb
    pop rdi
    mov rax, r9
    ret
.globl memcmp
.globl bcmp
bcmp:
memcmp:
    xor eax, eax
    test r8, r8
    jz memcmp_end
memcmp_loop:
    movzx eax, byte ptr [rcx]
    movzx r9d, byte ptr [rdx]
    sub eax, r9d
    jnz memcmp_end
    inc rcx
    inc rdx
    dec r8
    jnz memcmp_loop
memcmp_end:
    ret
.globl wcslen
wcslen:
    xor eax, eax
wcslen_loop:
    cmp word ptr [rcx + rax*2], 0
    je wcslen_end
    inc rax
    jmp wcslen_loop
wcslen_end:
    ret
.globl __chkstk
__chkstk:
    push rcx
    push rax
    lea rcx, [rsp + 24]
    cmp rax, 0x1000
    jb chkstk_last
chkstk_loop:
    sub rcx, 0x1000
    test byte ptr [rcx], 0
    sub rax, 0x1000
    cmp rax, 0x1000
    ja chkstk_loop
chkstk_last:
    sub rcx, rax
    test byte ptr [rcx], 0
    pop rax
    pop rcx
    ret

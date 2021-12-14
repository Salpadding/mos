org 0x7c00
bits 16

; initialize segments
mov ax, 0
mov ds, ax
mov es, ax
mov ss, ax

mov al, 0
or  al, 0
jz b_1

b_0:
    mov ax, 1
b_1:
    mov ax, 0
times 510-($-$$) db 0
dw 0AA55h
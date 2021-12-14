org 0x7c00

; initialize segments
mov ax, 0
mov ds, ax
mov es, ax
mov ss, ax
mov sp, 0x7c00

call cls
mov si, welcome
call puts
jmp $

; clearn screen
; fill from 0xb8000 to 0xbffff with blank
cls:
    mov ax, 0xb000
    mov gs, ax
    mov bx, 0x8000
    jmp .lp
.lp:
    mov byte gs:[bx], ' '
    add bx, 1
    cmp bx, 0xffff  
    jz  .done
    mov byte gs:[bx], 0xf
    add bx, 1
    cmp bx, 0xffff
    jz  .done
    jmp .lp
.done:
    ret

puts:
    mov ax, 0xb800
    mov gs, ax
    mov bx, 0
    jmp .lp
.lp:
    lodsb
    or al, al
    jz .done
    mov gs:[bx], al
    add bx, 1
    mov byte gs:[bx], 0xf
    add bx, 1
    jmp .lp
.done:
    ret

welcome db 'Loading...' , 0

times 510-($-$$) db 0
dw 0AA55h
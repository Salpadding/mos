%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR
; clear screent
call cls

push loading
pre 
call puts
rt

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

; put string
puts:
    mov si, [bp + 2]
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

loading db 'Loading...' , 0

times 8192 db 0
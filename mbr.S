%macro pre 0
    push bp
    mov bp, sp
%endmacro

%macro rt 0
    mov sp, bp
    pop bp
%endmacro

org 0x7c00

; initialize segments
mov ax, 0
mov ds, ax
mov es, ax
mov ss, ax
mov sp, 0x7c00
mov bp, sp


call cls

push 0x9000
push 1
push 1
pre
call read_n_sec
rt

push 0x9000
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

; read_n_sec(start_sec, n, dst)
; bp + 2, bp + 4, bp + 6
read_n_sec:
    ; set sector n
    mov dx, 0x1f2
    mov ax, [bp + 4]
    out dx, al

    ; set sector number
    mov dx, 0x1f3
    mov ax, [bp + 2]
    out dx, al

    mov dx, 0x1f4
    mov al, ah
    out dx, al

    mov dx, 0x1f5
    mov al, 0
    out dx, al

    ; enable lba mode
    mov dx, 0x1f6
    mov al, 0xe0
    out dx, al

    ; read command
    mov dx, 0x1f7
    mov al, 0x20
    out dx, al

; wait until ready
.ready:
    nop
    in al, dx
    and al, 0x88
    cmp al, 0x08
    jnz .ready

; read from 0x1f0
; set loop count
    mov ax, [bp + 4]
    mov dx, 256
    mul dx
    mov cx, ax
    mov dx, 0x1f0
    mov bx, [bp + 6]

.read:
    in ax, dx
    mov [bx], ax
    add bx, 2
    loop .read
    ret


load0 db 'Loading...' , 0

times 510-($-$$) db 0
dw 0AA55h
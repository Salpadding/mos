%include "boot.inc"
SECTION MBR vstart=MBR_BASE_ADDR

; initialize segments
mov ax, 0
mov ds, ax
mov es, ax
mov ss, ax
mov sp, MBR_BASE_ADDR
mov bp, sp


push LOADER_BASE_ADDR
push LOADER_SECTORS
push LOADER_START_SECTOR
pre
call read_n_sec
rt

jmp LOADER_BASE_ADDR

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


times 510-($-$$) db 0
db 0x55
db 0xaa
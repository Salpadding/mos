%include "boot.inc"
SECTION MBR vstart=MBR_BASE_ADDR

; initialize segments
mov ax, 0
mov ds, ax
mov es, ax
mov ss, ax
mov sp, MBR_BASE_ADDR
mov bp, sp


; load kernel loader
mov bx, LOADER_START_SECTOR
mov cx, LOADER_SECTORS
mov dx, LOADER_BASE_ADDR
call read_n_sec

; load kernel to [0x70000, )
mov ax, KERNEL_BASE_ADDR >> 4
mov ds, ax

mov bx, 1 + LOADER_SECTORS
mov cx, KERNEL_SECTORS
mov dx, 0
call read_n_sec

mov ax, 0
mov ds, ax

jmp LOADER_BASE_ADDR

; read_n_sec(start_sec, n, dst)
; read_n_sec(bx, cx, dx)
; start_sec <= 65535, n <= 65535
; bp + 2, bp + 4, bp + 6
read_n_sec:
    push bp
    mov  bp, sp

    push bx
    push cx
    push dx

    ; set sector n
    mov dx, 0x1f2
    mov ax, [bp - 4]
    out dx, al

    ; set sector number
    mov dx, 0x1f3
    mov ax, [bp - 2]
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
    mov eax, 0
    mov ax, [bp - 4]
    mov edx, 256
    mul edx
    mov ecx, eax
    mov dx, 0x1f0
    mov bx, [bp - 6]

.read:
    in ax, dx
    mov [bx], ax
    add bx, 2
    cmp bx, 0
    jnz .con
.seg:
    mov ax, ds
    add ax, 0x1000
    mov ds, ax
.con:
    loop .read, ecx
    pop dx
    pop cx
    pop bx
    pop bp
    ret


times 510-($-$$) db 0
db 0x55
db 0xaa
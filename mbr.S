%include "boot.inc"
SECTION MBR vstart=MBR_BASE_ADDR

; initialize segments
mov ax, 0
mov ds, ax
mov es, ax
mov ss, ax
mov sp, LOADER_BASE_ADDR - 8
mov bp, sp


; load kernel loader
mov bx, LOADER_START_SECTOR
mov cx, LOADER_SECTORS
mov dx, LOADER_BASE_ADDR
call read_n_sec

mov ax, KERNEL_BASE_ADDR >> 4
mov ds, ax


mov ax, 0
mov ds, ax

mov ebx, 0
jmp LOADER_BASE_ADDR

mc_read_n_sec dx, si, bx

times 510-($-$$) db 0
db 0x55
db 0xaa
MBR_BASE_ADDR equ 0x7c00
LOADER_BASE_ADDR equ 0x800
LOADER_START_SECTOR equ 1
LOADER_SECTORS equ 16
KERNEL_SECTORS equ 124
KERNEL_ENTRY equ 0x100000

; read_n_sec(start_sec, n, dst)
; read_n_sec(bx, cx, dx)
; read_n_sec(bp - 2, bp - 4, bp - 6)
; start_sec <= 65535, n <= 65535
; mc_read_n_sec dx, si, bx
; mc_read_n_sec edx, esi, ebx
%macro mc_read_n_sec 3
read_n_sec:
    ; backup dx/edx into si/esi
    mov %2, %1
    ; backup sectors
    mov di, cx

    ; set sector n
    mov dx, 0x1f2
    mov ax, cx
    out dx, al

    ; set sector number
    mov dx, 0x1f3
    mov ax, bx,
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
    mov ax, di
    mov edx, 256
    mul edx
    mov ecx, eax
    mov dx, 0x1f0
    mov %3, %2

.read:
    in ax, dx
    mov [%3], ax
    add %3, 2
    cmp %3, 0
    jnz .con
.seg:
    mov ax, ds
    add ax, 0x1000
    mov ds, ax
.con:
    loop .read, ecx
    ret


%endmacro
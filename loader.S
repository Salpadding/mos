%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR

; enter protect mode
jmp protect

; segment descriptors
gdt_base:
    dq 0 

; define a code segment limit = 0xfffff * 4kb = 4G, only executable
; base = 0
code_desc:
    ; limit 0-15
    dw 0xFFFF
    ; base 0-15
    dw 0
    ; base 16-23
    db 0 

    ; access byte
    ; bit[3]: executable 
    ; bits[4]: defines a code or data segment
    ; bits[7]: must be 1
    db 10011000b

    ; flags and limit
    ; bits[0] ~ bits[3] limit
    ; bits[4]: reserved
    ; bits[5]: long mode, for 64bit
    ; bits[6]: 32bit
    ; bits[7]: page size = 4kb
    db 0xcf
    ; base 23-31
    db 0



; define a data segment limit = 0xfffff * 4kb = 4G, read/writable, while not executable
; base = 0
data_stack_desc:
    ; limit 0-15
    dw 0xFFFF
    ; base 0-15
    dw 0
    ; base 16-23
    db 0 

    ; access byte
    ; bits[1]: writable
    ; bits[4]: defines a code or data segment
    db 10010010b

    ; flags and limit
    ; bits[0] ~ bits[3] limit
    ; bits[4]: reserved
    ; bits[5]: long mode, for 64bit
    ; bits[6]: 32bit
    ; bits[7]: page size = 4kb
    db 0xcf
    ; base 23-31
    db 0

; define a data segment limit = 8 * 4kb = 32kb, base = 0xb8000 read/writable, while not executable
; base = 0
data_vedio_desc:
    ; limit 0-15
    dw 8
    ; base 0-15
    dw 0x8000

    ; base 16-23
    db 0x0b 

    ; access byte
    ; bits[1]: writable
    ; bits[4]: defines a code or data segment
    db 10010010b

    ; flags and limit
    ; bits[0] ~ bits[3] limit
    ; bits[4]: reserved
    ; bits[5]: long mode, for 64bit
    ; bits[6]: 32bit
    ; bits[7]: page size = 4kb
    db 0xc0
    ; base 23-31
    db 0

gdt_end:

SELECTOR_CODE   equ (1 << 3)
SELECTOR_DATA   equ (2 << 3)
SELECTOR_VIDEO  equ (3 << 3)

gdt_ptr:
    dw gdt_end - gdt_base - 1
    dd gdt_base

; jump into protection mode
protect:
    mov sp, LOADER_BASE_ADDR
    mov bp, sp

    ; a20
    in al, 0x92
    or al, 0x02
    out 0x92, al

    cli
    ; load gdt
    lgdt [gdt_ptr]

    ; cr0 pe
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ; set cs = gdt[1], ip = p_mode_start
    jmp SELECTOR_CODE:p_mode_start

; clearn screen
; fill from 0xb8000 to 0xbffff with blank
cls:
    mov ax, 0xb000
    mov gs, ax
    mov bx, 0x8000
    jmp .lp
.lp:
    mov byte [gs:bx], ' '
    add bx, 1
    cmp bx, 0xffff  
    jz  .done
    mov byte [gs:bx], 0xf
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
    mov [gs:bx], al
    add bx, 1
    mov byte [gs:bx], 0xf
    add bx, 1
    jmp .lp
.done:
    ret

loading db 'Loading...' , 0
msg db 'Protect mode...' , 0

[bits 32]
; clear screen in protect mode
cls_p: 
    mov ax, SELECTOR_VIDEO
    mov gs, ax
    mov ebx, 0
    jmp .lp
.lp:
    mov byte [gs:ebx], ' '
    add ebx, 1
    cmp ebx, 0x7fff
    jz  .done
    mov byte [gs:ebx], 0xf
    add ebx, 1
    cmp ebx, 0x7fff
    jz  .done
    jmp .lp
.done:
    ret

puts_p:
    mov esi, [ebp + 4]
    mov ax, SELECTOR_VIDEO
    mov gs, ax
    mov ebx, 0
    jmp .lp
.lp:
    lodsb
    nop
    or al, al
    jz .done
    mov byte [gs:ebx], al
    add ebx, 1
    mov byte [gs:ebx], 0xf
    add ebx, 1
    jmp .lp
.done:
    ret

p_mode_start:
    ; initialize data, stack base
    mov ax, SELECTOR_DATA
    mov ds, ax
    mov es, ax
    mov ss, ax

    mov esp, LOADER_BASE_ADDR
    mov ebp, esp

    call cls_p

    push msg
    pp
    call puts_p
    rp

    jmp $


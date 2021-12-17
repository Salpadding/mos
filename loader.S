%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR

; create page directory entry at i with attribute 
; allocate and map
; %1 = pde index, %2 = attribute, %3 = page table address
%macro create_pde 3
    mov eax, 4
    mov ebx, %1
    mul ebx
    add eax, 0x10000
    mov ebx, eax

    mov ecx, 0
    or  ecx, %2
    or  ecx, %3
    mov [ebx], ecx
%endmacro


%macro init_pte
    
%endmacro

; enter protect mode
jmp start

; memory allocation of pde and pte
; 0 <= i < 1024 pde(i) address = 0x10000 + i * 4
; for 0 <= i < 1024, pde(i):
; 0 <= j < 1024 pte(j) address = 0x10000 + (i+1) * 4096 + j * 4


setup_page:
    ; clear [0x10000, 0x10000 + 4096)
    mov ecx, 4096
    mov ebx, 0x10000
.clear:
    mov byte [ebx], 0
    inc ebx
    loop .clear
.pde:
    create_pde 0,   7, 0x10000 + 4096 * 1
    create_pde 768, 7, 0x10000 + 4096 * 1
    create_pde 1023,7, 0x10000

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

mem_size:
    ; get memory size
    mov ax, 0xe801
    int 0x15

    ; ax * 1024 + bx * 1024 * 64 + 1024 * 1024

    ; esp + 8
    push ebx;
    ; esp + 4
    push eax

    ; ax * 1024
    mov eax, [esp]
    mov ebx, 1024
    mul ebx
    mov [esp], eax

    mov eax, [esp + 4]
    mul ebx
    mov ebx, 64
    mul ebx
    mov [esp + 4], eax

    mov eax, 1024
    mul eax
    add eax, [esp]
    add eax, [esp + 4]

    add esp, 8
    ret



; jump into protection mode
start:
    mov sp, LOADER_BASE_ADDR
    mov bp, sp

    call mem_size

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
    mov ecx, 0x4000
.lp:
    mov word [gs:ebx], 0x0f20
    add ebx, 2
    loop .lp
    ret

puts_p:
    mov esi, [ebp - 4]
    mov ax, SELECTOR_VIDEO
    mov gs, ax
    mov ebx, 0
    jmp .lp
.lp:
    lodsb
    mov ah, 0xf
    or al, al
    jz .done
    mov [gs:ebx], ax
    add ebx, 2
    jmp .lp
.done:
    ret

; fibo(a) = fibo(a - 1) + fibo(a - 2)
; fibo(0) = 0
; fibo(1) = 1
fibo:
    ; allocate two local variable
    sub esp, 4
    mov eax, [ebp - 4]
    cmp eax, 0
    jz  .r0
    cmp eax, 1
    jz  .r1

    ; get fibo(a - 1)
    pp 
    sub eax, 1
    push eax
    call fibo
    add esp, 4
    rp

    ; store return value
    mov [ebp - 12], eax

    ; get fibo(a - 2)
    mov eax, [ebp - 4]
    pp
    sub eax, 2
    push eax
    call fibo
    add esp, 4
    rp

    add eax, [ebp - 12]
    add esp, 4
    ret
.r0:
    add esp, 4
    mov eax, 0
    ret
.r1:
    add esp, 4
    mov eax, 1
    ret

; fact(a) = a * fact(a-1)
fact:
    ; arguments
    mov eax, [esp + 4]
    ; cmp a and zero
    cmp eax, 0
    jz .done

    ; get fact(a - 1)
    sub eax, 1

    pp 
    push eax
    call fact
    add esp, 4
    rp

    mov ebx, eax
    mov eax, [esp + 4]
    mul ebx
    ret
.done:
    ; fact(0) = 1
    mov eax, 1
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

    pp
    push msg
    ; ebp + 4 = msg
    call puts_p
    add esp, 4
    rp

    jmp $


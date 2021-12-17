%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR

; for 0 <= i < 1024 pde(i) address = 0x10000 + i * 4
; create page directory entry at i with attribute 
; allocate and map
; %1 = pde index, %2 = attribute, %3 = page table address
%macro create_pde 3
    mov eax, 4
    mov ebx, %1
    mul ebx
    add eax, 0x10000
    mov ebx, eax

    mov eax, 0
    or  eax, %2
    or  eax, %3
    mov [ebx], eax
%endmacro

; %1 = pte address, %2 = physical address start, %3 = page count, %4 = attribute
;  esp + 4           , esp + 8                    ,  esp + 12  , esp + 16
mmap:
    ; loop n pages
    mov ecx, [esp + 12]
    ; pte[0] physical address, often starts with 0x10000 + 4096
    mov ebx, [esp + 4]
    ; physical address
    mov eax, [esp + 8]
.cnt
    mov edx, eax
    ; set attribute
    or edx, [esp + 16]
    mov [ebx], edx
    add ebx, 4
    add eax, 4096
    loop .cnt
    ret


; %1 = i, %2 = j, %3 = attribute %4 = addres
%macro init_pte 4
    mov eax, 4096
    mov ebx, %1
    inc ebx
    mul ebx
    add eax, 0x10000

    push eax

    mov eax, %2
    mov ebx, 4
    mul ebx
    add [esp], eax

    mov ebx, [esp]
    mov eax, %3
    or eax, %4
    mov [ebx], eax
    add esp, 4


%endmacro

; enter protect mode
jmp start


; mapping from virtual memory x to physical memory y: 
; address of pde[0] is 0x100000
; pde index i = (x & 0xffc00000) >> 22
; address of pde[i] = 0x100000 + 4 * i
; pte index j = (x & 0x3ff000) >> 12
; address of pte[0] = pde[i] & 0xfffff000
; address of pte[j] = &pte[0] + 4 * j
; address of page = pte[j] & 0xfffff000
; y = (pte[j] & 0xfffff000) + (y & 0xfff)

setup_page:
    mov ecx, 4096
    mov ebx, 0x10000
.clear:
    mov byte [ebx], 0
    inc ebx
    loop .clear
.pde:
    ; let [0, 1mb) in virtual map to physical [0, 1M)
    create_pde 0,   7, 0x10000 + 4096 * 1
    ; let [3G, 3G+ 1mb) in virtual map to physical [0, 1M)
    create_pde 768, 7, 0x10000 + 4096 * 1
    create_pde 1023,7, 0x10000

    ; attribute
    push dword 7
    ; page count
    push dword 256
    ; physical address start
    push dword 0
    ; pte start address
    push dword 0x10000 + 4096 * 1
    call mmap
    add esp, 16


.pte:
    ; initialized page table entry from 
    init_pte 0, esi, 7, 0
    loop .pte


    ; create pde for pde [769, 1023)
    mov esi, 769
    mov edx, 0x10000
    add edx, 0x2000
    mov ecx, 1023 - 769
.kernel:
    create_pde esi, 7, edx
    inc esi
    add edx, 0x1000
    loop .kernel



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


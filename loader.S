%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR
PDE_START equ 0x100000

; enter protect mode
jmp start

; pdes[i] = %2 || %3
; for 0 <= i < 1024 pde(i) address = 1M + i * 4
; create page directory entry at i with attribute 
; allocate and map
; %1 = pde index, %2 = attribute, %3 = page table address
%macro create_pde 3
    mov eax, 4
    mov ebx, %1
    mul ebx
    add eax, PDE_START
    mov ebx, eax

    mov eax, 0
    or  eax, %2
    or  eax, %3
    mov [ebx], eax
%endmacro


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

gdt_end:

SELECTOR_CODE   equ (1 << 3)
SELECTOR_DATA   equ (2 << 3)

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

    ; setup gdt

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
    mov ebx, 0xc00b8000
    mov ecx, 0x4000
.lp:
    mov word [ebx], 0x0f20
    add ebx, 2
    loop .lp
    ret

puts_p:
    mov ebx, 0xc00b8000
    mov esi, [ebp - 4]
    jmp .lp
.lp:
    lodsb
    mov ah, 0xf
    or al, al
    jz .done
    mov [ebx], ax
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


    ; enable paging
    call setup_page
    mov  eax, PDE_START
    mov  cr3, eax

    mov eax, cr0
    or  eax, 1 << 31
    mov cr0, eax

    ; increase stack base with 3GB
    add esp, 0xc0000000

    call cls_p

    pp
    push msg
    ; ebp + 4 = msg
    call puts_p
    add esp, 4
    rp

    call kernel_init

    jmp SELECTOR_CODE:KERNEL_ENTRY


; %1 = pte address, %2 = physical address start, %3 = page count, %4 = attribute
;  esp + 4           , esp + 8                    ,  esp + 12  , esp + 16
mmap:
    ; loop n pages
    mov ecx, [esp + 12]
    ; pte[0] physical address, often starts with 0x10000 + 4096
    mov ebx, [esp + 4]
    ; physical address
    mov eax, [esp + 8]
.cnt:
    mov edx, eax
    ; set attribute
    or edx, [esp + 16]
    mov [ebx], edx
    add ebx, 4
    add eax, 4096
    loop .cnt
    ret


; mapping from virtual memory x to physical memory y: 
; address of pde[0] is 0x100000
; pde index i = (x & 0xffc00000) >> 22
; address of pde[i] = 0x100000 + 4 * i
; pte index j = (x & 0x3ff000) >> 12
; address of pte[0] = pde[i] & 0xfffff000
; address of pte[j] = &pte[0] + 4 * j
; address of page = pte[j] & 0xfffff000
; y = (pte[j] & 0xfffff000) + (y & 0xfff)


; address page table info it self
; for ith pde
; &pde[i] == (1023 << 22) | (1023 << 12)  | (i * 4)
get_pde:
    mov eax, [esp + 4]
    shl eax, 2
    add eax, (1023 << 22) | (1023 << 12)
    mov eax, [eax]
    ret

; i = esp + 4, j = esp + 8
get_pte:
    mov eax, [esp + 4]
    shl eax, 12
    add eax, 1023 << 22
    mov ebx, eax
    mov eax, [esp + 8]
    shl eax, 2
    add eax, ebx
    mov eax, [eax]
    ret

setup_page:
    ; for(int i = 0; i < 1024; i++) pdes[i] = 0
    mov ecx, 4096
    mov ebx, PDE_START
.clear:
    mov byte [ebx], 0
    inc ebx
    loop .clear
.pde:
    ; let [0, 1mb) in virtual map to physical [0, 1M)
    ; pdes[0] = 7 || pdes + 4096
    create_pde 0,   7, PDE_START + 4096 * 1
    ; let [3G, 3G+ 1mb) in virtual map to physical [0, 1M)
    ; pdes[768] = pdes[0]
    create_pde 768, 7, PDE_START + 4096 * 1
    ; pdes[1023] = 7 | pdes
    create_pde 1023,7, PDE_START

    ; create 256 pte 
    ; attribute
    push dword 7
    ; page count
    push dword 256
    ; physical address start
    push dword 0
    ; pte start address
    push dword PDE_START + 4096 * 1
    call mmap
    add esp, 16
    ret

; void kernel_init();
kernel_init:
    ; allocate three local variable
    push ebp
    mov ebp, esp
    sub esp, 12

    mov ebx, KERNEL_BASE_ADDR 
    mov esi, 28

    ; [ebp - 4] offset of headers
    mov eax, [ebx + esi]
    mov dword [ebp - 4], eax

    mov esi, 44
    mov eax, 0
    mov ax,  [ebx + esi]
    ; [ebp - 8] number of headers
    mov dword [ebp - 8], eax

    ; [ebp - 12] current header index
    mov eax, 0
    mov [ebp - 12], eax
.loop:
    mov eax, 32
    mul dword [ebp - 12] 
    add eax,  [ebp - 4]
    add eax,  KERNEL_BASE_ADDR
    ; continue only if type == 1
    cmp dword [eax], 1
    jnz .cond

    ; call memcpy
    mov ebx, eax

    pp
    ; size
    push dword [ebx + 16]

    ; source
    mov eax, [ebx + 4]
    add eax, KERNEL_BASE_ADDR
    push eax

    ; dest
    push dword [ebx + 8]
    call mem_cpy
    add  esp, 12
    rp

.cond:
    add dword [ebp - 12], 1
    mov eax, [ebp - 8]
    cmp [ebp - 12], eax
    jz .done
    jmp .loop

.done:
    add esp, 12
    pop ebp
    ret

; mem_cpy(dst, src, size)
;         -12 , -8, -4
mem_cpy:
    cld
    ; backup ecx
    push ecx
    mov esi, [ebp - 8]
    mov edi, [ebp - 12]
    mov ecx, [ebp - 4]
    rep movsb
    pop ecx
    ret

    
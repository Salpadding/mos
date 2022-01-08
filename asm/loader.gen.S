[bits 16]
%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR
ASM_BUF_OFF equ 4096
ASM_BUF_LEN equ 256

jmp start
times 8 - ($ - $$) db 0

[bits 32]
gdt_base:
    times 8 dq 0 
gdt_end:

times ASM_BUF_OFF - LOADER_BASE_ADDR - ($ - $$) db 0
; 4 argument for asm_api call, 16 byte
asm_buf:
    times ASM_BUF_LEN db 0

; loader_api, 1 = gdt_ptr, 2 = lidt, 3 = page enabled?
asm_api:
    mov eax, [asm_buf]
    ; gdt ptr
    cmp eax, 0
    jz .gdt
    cmp eax, 1
    jz .int_entries
    cmp eax, 2
    jz .int_rust
    cmp eax, 3
    jz .mem_sz
    cmp eax, 4
    jz .switch
    cmp eax, 5
    jz .int_exit
    cmp eax, 6
    jz .sys
    mov eax, 0
    ret
.gdt:
    mov dword [asm_buf], gdt_ptr
    ret
.int_entries:
    mov dword [asm_buf], int_entries
    ret
.int_rust:
    mov dword [asm_buf], int_rust
    ret
.mem_sz:
    mov eax, [mem_sz],
    mov [asm_buf], eax
    ret
.switch:
    mov dword [asm_buf], switch
    ret
.int_exit:
    mov dword [asm_buf], int_exit
    ret
.sys:
    mov dword [asm_buf], sys
    ret
mem_sz:
    dd 0

mc_read_n_sec edx, esi, ebx

[bits 16]

SELECTOR_CODE   equ (1 << 3)
SELECTOR_DATA   equ (2 << 3)

gdt_ptr:
    dw gdt_end - gdt_base - 1
    dd gdt_base

get_mem_size:
    ; get memory size
    mov ax, 0xe801
    int 0x15

    push bp
    ; ax * 1024 + bx * 1024 * 64 + 1024 * 1024

    ; bp + 4
    push ebx;
    ; bp
    push eax

    mov  bp, sp

    ; ax * 1024
    mov eax, [bp]
    mov ebx, 1024
    mul ebx
    mov [bp], eax

    mov eax, [bp + 4]
    mul ebx
    mov ebx, 64
    mul ebx
    mov [bp + 4], eax

    mov eax, 1024
    mul eax
    add eax, [bp]
    add eax, [bp + 4]

    add sp, 8
    pop bp
    ret

; jump into protection mode
start:
    call get_mem_size
    mov  [mem_sz], eax

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

[bits 32]
p_mode_start:
    ; initialize data, stack base
    mov ax, SELECTOR_DATA
    mov ds, ax
    mov es, ax
    mov ss, ax

    mov ebx, 1 + LOADER_SECTORS
    mov ecx, KERNEL_SECTORS
    mov edx, KERNEL_ENTRY
    call read_n_sec

    jmp SELECTOR_CODE:KERNEL_ENTRY

%define ERROR_CODE nop		 ; 若在相关的异常中cpu已经自动压入了错误码,为保持栈中格式统一,这里不做操作.
%define ZERO push dword 0		 ; 若在相关的异常中cpu没有压入错误码,为了统一栈中格式,就手工压入一个0

%macro VECTOR 2
int_%1_entry:		 ; 每个中断处理程序都要压入中断向量号,所以一个中断类型一个中断处理程序，自己知道自己的中断向量号是多少
   %2				 ; 中断若有错误码会压在eip后面
   push ds ; save context
   push es
   push fs
   push gs
   pushad

   ; 如果是从片上进入的中断,除了往从片上发送EOI外,还要往主片上发送EOI
   mov al,0x20                   ; 中断结束命令EOI
   out 0xa0,al                   ; 向从片发送
   out 0x20,al                   ; 向主片发送

   push dword %1
   push esp
   call [int_rust]       ; call rust interrupt handler
   add  esp, 4
   jmp int_exit

%endmacro

; insert idt code after this line
int_entries:
   dd int_0x00_entry
   dd int_0x01_entry
   dd int_0x02_entry
   dd int_0x03_entry
   dd int_0x04_entry
   dd int_0x05_entry
   dd int_0x06_entry
   dd int_0x07_entry
   dd int_0x08_entry
   dd int_0x09_entry
   dd int_0x0a_entry
   dd int_0x0b_entry
   dd int_0x0c_entry
   dd int_0x0d_entry
   dd int_0x0e_entry
   dd int_0x0f_entry
   dd int_0x10_entry
   dd int_0x11_entry
   dd int_0x12_entry
   dd int_0x13_entry
   dd int_0x14_entry
   dd int_0x15_entry
   dd int_0x16_entry
   dd int_0x17_entry
   dd int_0x18_entry
   dd int_0x19_entry
   dd int_0x1a_entry
   dd int_0x1b_entry
   dd int_0x1c_entry
   dd int_0x1d_entry
   dd int_0x1e_entry
   dd int_0x1f_entry
   dd int_0x20_entry
   dd int_0x21_entry
   dd int_0x22_entry
   dd int_0x23_entry
   dd int_0x24_entry
   dd int_0x25_entry
   dd int_0x26_entry
   dd int_0x27_entry
   dd int_0x28_entry
   dd int_0x29_entry
   dd int_0x2a_entry
   dd int_0x2b_entry
   dd int_0x2c_entry
   dd int_0x2d_entry
   dd int_0x2e_entry
   dd int_0x2f_entry

int_rust:
   dd 0

VECTOR 0x00, ZERO
VECTOR 0x01, ZERO
VECTOR 0x02, ZERO
VECTOR 0x03, ZERO
VECTOR 0x04, ZERO
VECTOR 0x05, ZERO
VECTOR 0x06, ZERO
VECTOR 0x07, ZERO
VECTOR 0x08, ERROR_CODE
VECTOR 0x09, ZERO
VECTOR 0x0a, ERROR_CODE
VECTOR 0x0b, ERROR_CODE
VECTOR 0x0c, ZERO
VECTOR 0x0d, ERROR_CODE
VECTOR 0x0e, ERROR_CODE
VECTOR 0x0f, ZERO
VECTOR 0x10, ZERO
VECTOR 0x11, ERROR_CODE
VECTOR 0x12, ZERO
VECTOR 0x13, ZERO
VECTOR 0x14, ZERO
VECTOR 0x15, ZERO
VECTOR 0x16, ZERO
VECTOR 0x17, ZERO
VECTOR 0x18, ERROR_CODE
VECTOR 0x19, ZERO
VECTOR 0x1a, ERROR_CODE
VECTOR 0x1b, ERROR_CODE
VECTOR 0x1c, ZERO
VECTOR 0x1d, ERROR_CODE
VECTOR 0x1e, ERROR_CODE
VECTOR 0x1f, ZERO
VECTOR 0x20, ZERO
VECTOR 0x21, ZERO
VECTOR 0x22, ZERO
VECTOR 0x23, ZERO
VECTOR 0x24, ZERO
VECTOR 0x25, ZERO
VECTOR 0x26, ZERO
VECTOR 0x27, ZERO
VECTOR 0x28, ZERO
VECTOR 0x29, ZERO
VECTOR 0x2a, ZERO
VECTOR 0x2b, ZERO
VECTOR 0x2c, ZERO
VECTOR 0x2d, ZERO
VECTOR 0x2e, ZERO
VECTOR 0x2f, ZERO
;;; IDT_CODE ;;;

int_exit:
   add esp, 4
   popad
   pop gs
   pop fs
   pop es
   pop ds
   add esp, 4
   iretd

switch: ; switch(cur, next)
    ; esp = ret, esp + 4 = cur, esp + 8 = next 
    push ds
    push es
    push fs
    push gs
    pushad
    ; esp + 32 = ret, esp + 36 = cur, esp + 40 = next 

    mov eax, [esp + 52]    
    mov [eax], esp ; save current context
    ; switch to next context

    mov eax, [esp + 56]
    mov esp, [eax]

    popad
    pop gs
    pop fs
    pop es
    pop ds
    ret

sys:
   push dword 0
   push ds
   push es
   push fs
   push gs
   pushad


   push dword 0x80
   push esp
   call [int_rust]       ; call rust interrupt handler
   add  esp, 4
   jmp int_exit


times 8*1024 - ($ - $$) db 0 
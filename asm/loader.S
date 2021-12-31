[bits 16]
%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR
ASM_BUF_OFF equ 4096
ASM_BUF_LEN equ 256

jmp start
times 8 - ($ - $$) db 0

[bits 32]
gdt_base:
    dq 0
    dq 0
    dq 0
    dq 0
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
   call [int_rust]       ; call rust interrupt handler
   add  esp, 4
   jmp int_exit

%endmacro

; insert idt code after this line
;;; IDT_CODE ;;;

int_exit:
   popad
   pop gs
   pop fs
   pop es
   pop ds
   add esp, 4
   iretd

switch: ; switch(cur, next)
    ; esp = ret, esp + 4 = cur, esp + 8 = next 
    push esi
    push edi
    push ebx
    push ebp
    ; esp + 16 = ret, esp + 20 = cur, esp + 24 = next 

    mov eax, [esp + 20]    
    mov [eax], esp ; save current context
    ; switch to next context

    mov eax, [esp + 24]
    mov esp, [eax]
    pop ebp
    pop ebx
    pop edi
    pop esi
    ret

times 8*1024 - ($ - $$) db 0 
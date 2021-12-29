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

; store esp temporary in isr
int_esp_0:
    dd 0
int_esp_1:
    dd 0

%define ERROR_CODE nop		 ; 若在相关的异常中cpu已经自动压入了错误码,为保持栈中格式统一,这里不做操作.
%define ZERO push dword 0		 ; 若在相关的异常中cpu没有压入错误码,为了统一栈中格式,就手工压入一个0

%macro VECTOR 2
int_%1_entry:		 ; 每个中断处理程序都要压入中断向量号,所以一个中断类型一个中断处理程序，自己知道自己的中断向量号是多少
   %2				 ; 中断若有错误码会压在eip后面
   mov  [int_esp_0], esp ; backup esp

; 以下是保存上下文环境
   mov  esp, asm_buf + ASM_BUF_LEN

   ; save eax, ecx, edx, ebx, esp, ebp, esi, edi
   ; and ds, es, fs, gs
   pushad
   mov eax, [int_esp_0]
   add eax, 16
   mov [esp + 12], eax

   ; we are now 32bit
   push ds
   push es
   push fs
   push gs

   mov [int_esp_1], esp
   mov esp, [int_esp_0]

   ; error code
   pop dword [asm_buf + 4]

   ; eip
   pop dword [asm_buf + 8]

   ; cs
   pop dword [asm_buf + 12]

   ; eflags
   pop dword [asm_buf + 16]

   ; 如果是从片上进入的中断,除了往从片上发送EOI外,还要往主片上发送EOI
   mov al,0x20                   ; 中断结束命令EOI
   out 0xa0,al                   ; 向从片发送
   out 0x20,al                   ; 向主片发送

   mov  dword [asm_buf], %1
   mov  eax, 0
   mov  ebx, eax
   mov  ecx, eax
   mov  edx, eax
   mov  esi, eax
   mov  edi, eax
   call [int_rust]       ; call rust interrupt handler
   mov [asm_buf], eax

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
;;; IDT_CODE ;;;

int_exit:
   mov esp, [int_esp_1]
   pop gs
   pop fs
   pop es
   pop ds
   popad

   mov esp, [asm_buf + ASM_BUF_LEN - 4 * 5]
   ; eflags
   push dword [asm_buf + 16]
   ; cs
   push dword [asm_buf + 12]
   ; eip
   push dword [asm_buf + 8]

   iretd

; align to 4k
; nasm will do some magic optimize if not reserve padding
times 10*1024 - ($ - $$) db 0 
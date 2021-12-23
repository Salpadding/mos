[bits 16]
%include "boot.inc"
SECTION LOADER vstart=LOADER_BASE_ADDR
PDE_START equ 0x100000

jmp start
times 8 - ($ - $$) db 0

[bits 32]
gdt_base:
    dq 0
    dq 0
    dq 0
    dq 0
gdt_end:

; 4 argument for asm_api call, 16 byte
asm_buf:
    dd 0
    dd 0
    dd 0
    dd 0

; loader_api, 1 = gdt_ptr, 2 = lidt, 3 = page enabled?
asm_api:
    mov eax, [asm_buf]
    cmp eax, 0
    jz .echo
    ; gdt ptr
    cmp eax, 1
    jz .gdt
    ; load interrupt descriptor table
    cmp eax, 2
    jz .lidt
    ; if page enabled
    cmp eax, 3
    jz .page_enabled
    ; setup page
    cmp eax, 4
    jz .set_page
    cmp eax, 5
    jz .int_entries
    cmp eax, 6
    jz .int_rust
    cmp eax, 7
    jz .div
    cmp eax, 8
    jz .caller
    cmp eax, 9
    jz .outb
    cmp eax, 10
    jz .outsw
    cmp eax, 11
    jz .inb
    cmp eax, 12
    jz .insw
    mov eax, 0
    ret
.echo:
    mov eax, [asm_buf + 4]
    mov [asm_buf], eax
    ret
.gdt:
    mov dword [asm_buf], gdt_ptr
    ret
.page_enabled:
    mov eax, [page_enabled]
    mov [asm_buf], eax
    ret
.set_page:
    ; the new stack base
    mov ecx, [asm_buf + 4]
    ; enable paging
    ; call setup_page
    mov  eax, PDE_START
    mov  cr3, eax

    mov eax, cr0
    or  eax, 1 << 31
    mov cr0, eax

    mov dword [page_enabled], 1

    mov eax, ecx
    mov esp, eax,
    mov ebp, esp

   call SELECTOR_CODE:KERNEL_ENTRY
.lidt:
    mov eax, [asm_buf + 4]
    lidt [eax]
    ret
.int_entries:
    mov dword [asm_buf], int_entries
    ret
.int_rust:
    mov dword [asm_buf], int_rust
    ret
.div:
    mov edx, 0
    mov eax, [asm_buf + 4]
    div dword [asm_buf + 8]
    mov [asm_buf], eax
    ret
.caller:
    mov eax, [esp]
    mov [asm_buf], eax
    ret
.outb:
    mov edx, [asm_buf + 4]
    mov eax, [asm_buf + 8]
    out dx, al
    ret
.outsw:
    cld
    mov edx, [asm_buf + 4]
    mov ecx, [asm_buf + 12]
    mov esi, [asm_buf + 8]
    rep outsw
    ret
.inb:
    mov edx, [asm_buf + 4]
    mov eax, 0
    in  al, dx
    mov [asm_buf], eax
    ret
.insw:
    cld
    mov edx, [asm_buf + 4]
    mov ecx, [asm_buf + 12]
    mov esi, [asm_buf + 8]
    rep insw
    ret


kernel_int_cb:
    dw 0

; interrupt handle entry
int_handle:
    iret

page_enabled:
    dd 0

mc_read_n_sec edx, esi, ebx

[bits 16]

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

    call SELECTOR_CODE:KERNEL_ENTRY

; align to 2k

%define ERROR_CODE nop		 ; 若在相关的异常中cpu已经自动压入了错误码,为保持栈中格式统一,这里不做操作.
%define ZERO push 0		 ; 若在相关的异常中cpu没有压入错误码,为了统一栈中格式,就手工压入一个0

%macro VECTOR 2
int_%1_entry:		 ; 每个中断处理程序都要压入中断向量号,所以一个中断类型一个中断处理程序，自己知道自己的中断向量号是多少

   %2				 ; 中断若有错误码会压在eip后面
; 以下是保存上下文环境
   push ds
   push es
   push fs
   push gs
   pushad			 ; PUSHAD指令压入32位寄存器,其入栈顺序是: EAX,ECX,EDX,EBX,ESP,EBP,ESI,EDI

   ; 如果是从片上进入的中断,除了往从片上发送EOI外,还要往主片上发送EOI
   mov al,0x20                   ; 中断结束命令EOI
   out 0xa0,al                   ; 向从片发送
   out 0x20,al                   ; 向主片发送

   push %1			 ;           ; push vector
   mov  dword [asm_buf], %1
   call [int_rust]       ; call rust interrupt handler

   jmp int_exit

%endmacro

; insert idt code after this line
;;; IDT_CODE ;;;

int_exit:
; 以下是恢复上下文环境
   add esp, 4			   ; 跳过中断号
   popad
   pop gs
   pop fs
   pop es
   pop ds
   add esp, 4			   ; 跳过error_code
   iretd

; align to 4k
; nasm will do some magic optimize if not reserve padding
times 4*1024 - ($ - $$) db 0 
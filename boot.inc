MBR_BASE_ADDR equ 0x7c00
LOADER_BASE_ADDR equ 0x9000
LOADER_START_SECTOR equ 1
LOADER_SECTORS equ 1

%macro pre 0
    push bp
    mov bp, sp
%endmacro

%macro rt 0
    mov sp, bp
    pop bp
    mov sp, bp
%endmacro

%macro pp 0
    push ebp
    mov  ebp, esp
%endmacro

%macro rp 0
    mov esp, ebp
    pop ebp
    mov esp, ebp
%endmacro
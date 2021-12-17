MBR_BASE_ADDR equ 0x7c00
LOADER_BASE_ADDR equ 0x9000
LOADER_START_SECTOR equ 1
LOADER_SECTORS equ 2

PAGE_DIR_CNT equ 1024
PAGE_DIR_BASE equ 0x10000
PAGE_DIR_SIZE equ PAGE_DIR_CNT * 4

%macro pre 0
    push bp
    mov bp, sp
%endmacro

%macro rt 0
    pop bp
    mov sp, bp
%endmacro

%macro pp 0
    push ebp
    mov  ebp, esp
%endmacro


%macro rp 0
    pop ebp
%endmacro
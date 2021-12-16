MBR_BASE_ADDR equ 0x7c00
LOADER_BASE_ADDR equ 0x9000
LOADER_START_SECTOR equ 1
LOADER_SECTORS equ 17
%macro pre 0
    push bp
    mov bp, sp
%endmacro

%macro rt 0
    mov sp, bp
    pop bp
    mov sp, bp
%endmacro
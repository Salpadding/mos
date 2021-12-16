MBR_BASE_ADDR equ 0x7c00
LOADER_BASE_ADDR equ 0x9000
LOADER_START_SECTOR equ 1
LOADER_SECTORS equ 2

%macro pre 0
    push bp
    mov bp, sp
%endmacro

%macro rt 0
    mov sp, bp
    pop bp
    mov sp, bp
%endmacro

; selector properties
RPL0  equ   00b
RPL1  equ   01b
RPL2  equ   10b
RPL3  equ   11b
TI_GDT	 equ   000b
TI_LDT	 equ   100b
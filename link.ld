SECTIONS {
    . =  0x200000;
    .entry  : { *(.entry) }
    .text   : { *(.text*) }      /* Excutable code                       */
    .rodata : { *(.rodata*) }    /* Constants (R/O)                      */
    .data   : { *(.data*) }      /* Initialized data                     */
    _data_end = .;              /* The end of .data section             */
    .bss    : { *(.bss*) }       /* Uninitialized data                   */
    _bss_end = .;               /* The end of .bss section              */
    /DISCARD/ : { *(.eh_frame*) }
}
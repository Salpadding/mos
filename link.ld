SECTIONS
{
  .text 0xc0001500 : {
    *(.init) /* _start */
    *(.text)
    *(.text.*)
  }
}
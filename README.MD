# MOS

A small kernel written in rust. 

## Memory allocation

1. 0-0x4ff              reserved
2. 0x500-0x800         stack area for mbr and kernel loader
3. 0x800-0x7bff        kernel loader, gdt, idt, asm api ( <29kb )
4. 0x7c00-0x7dff        mbr
5. 0x10000-0x0x7ffff    page directory (0x10000), reserved page mapping(0-8m->0-8m), (3g + 8m -> 0-8m), static area, for data structure before page setup (448kb)
6. 0x80000-0xfffff      reserved for hardware

--- (1MB)

5. 0x100000-0x4fffff    kernel (4MB)

--- (5MB)

7. 0x500000-0x800000    memory for os, heap, thread stack (3M)

--- (8MB)

## make.js

1. "link" program segments defined in kernel elf into kernel image
2. modify gdt in loader image

## Stages

1. mbr 

read loader image into memory, jump into loader


2. loader

- define asm api
- setup gdt, enter protected mode
- read kernel image, then execute 

3. kernel

- setup page table
- setup interrupt descriptor table


## Run

1. install bochs and nasm

```sh
brew install bochs
brew install nasm
```

2. generate hard disk

```sh
bximage
# 1
# hd
# flat
# 512
# 60
# build/disk.img
mkdir build
```

3. build bootloader and kernel

```sh
node make.js
```

4. run bochs

```sh
bochs
# return
# type c to continue
```


## Kernel initialization

1. initialize com1 port
2. initialize address of asm function switch and int exit (dynamic link)
3. initialize user privilege gdt and tss segment
4. initialize kernel memory pool (physical + virtual)  
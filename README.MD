# MOS

A small kernel written in rust. 

## Memory allocation

1. 0-0x4ff              reserved
2. 0x500-0x800         stack area for mbr and kernel loader
3. 0x800-0x7bff        kernel loader, gdt, idt, asm api ( <29kb )
4. 0x7c00-0x7dff        mbr
5. 0x7e00-0x0x7ffff    unused 
6. 0x80000-0xfffff      reserved for hardware

--- (1MB)

5. 0x100000-0x1fffff    page directory entry and page descriptor entries (1MB)


--- (2MB)

6. 0x200000-0x5ffff    kernal (4MB)

--- (6MB)


8. 0x600000-0x7fffff    stack area for kernal (2M)


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
mkdir build
bximage
# 1
# hd
# flat
# 512
# 60
# build/hd60M.img
```

3. build bootloader and kernel

```sh
./build.sh
```

4. run bochs

```sh
bochs
# return
# type c to continue
```

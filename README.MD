# MOS

## Memory allocation

1. 0-0x4ff              reserved
2. 0x500-0x7ff          stack area for mbr and kernel loader
3. 0x800-0x0x7ffff      kernel loader, gdt, idt, asm api (500kb)
4. 0x80000-0xfffff      reserved for hardware

--- (1MB)

5. 0x100000-0x2fffff    page directory entry and page descriptor entries (2MB)


--- (3MB)

6. 0x300000-0x6ffff    kernal (4MB)

--- (7MB)


8. 0x700000-0x7fffff    stack area for kernal (1M)


--- (8MB)


## Stages

1. mbr 

read loader image into memory, jump into loader


2. loader

- enter protected mode
- setup page table
- read kernel image, then execute 


3. kernel



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
# hd60M.img
```

3. compile boot loader

```sh
./debug.sh
```

4. run bochs

```sh
bochs
# return
# type c to continue
```

5. debug bootloader

```sh
vb 0:0x7c00 if 'cs == 0'
```

## Memory allocation

1. loader = 4kb, 8 sectors, starts from 0x500

## C language calling convention

```c
int add(int a, int b) {
    int c = a + b;
    return c;
}

int main() {
    return add(4, 5);
}
```

when calling add in main:

-- main
1. push 5
2. push 4
3. push return address
4. jump to add

-- add

4. push bp, bp points to stack top before (push5, push4) 
5. assign sp to bp, bp -> (5, 4, return address, original bp) 
6. do some arithmetic
7. assign bp to sp
8. pop and assign to bp
9. pop and jump 

## Assembly Notes


1. Moving an immediate into segment register is not allowed


2. x86_64 is Little-Endian

if memory = 

```py
memory = [0x11, 0x22, 0x33]
```

```asm
mov ax, 0
mov ds, ax
mov bx, [0]
```

then 

```
bx = 0x2211
```

3. Source in arithmetic operation is allowed to be immediate or memory address, destination in arithmetic is allowed to be memory address

```asm
add ax, 1
add ax, [0]


4. mov

Immediate is acceptable for only AX-DX and memory address.

- mov ax, 1
- mov ax, bx
- mov ax, ds
- mov ax, [0] 
- mov [0], ax
- mov ds, ax


5. Add, Sub

right: immediate, memory location, AX-DX
left: memory location, AX-DX

6. Push

right: immediate, memory location, AX-DX, SRegs

7. Mul

right: AX-DX, memory location
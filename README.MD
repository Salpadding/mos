# MOS

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




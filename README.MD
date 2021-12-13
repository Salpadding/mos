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
nasm -o mbr.bin mbr.S
dd if=mbr.bin of=hd60M.img bs=512 count=1 conv=notrunc
```

4. run bochs

```sh
bochs
# return
# type c to continue
```

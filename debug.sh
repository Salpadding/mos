#!/usr/bin/env zsh
nasm -o mbr.bin mbr.S
nasm -o loader.bin loader.S
dd if=mbr.bin of=hd60M.img bs=512 count=1 conv=notrunc
dd if=loader.bin of=hd60M.img bs=512 count=1 seek=1 conv=notrunc

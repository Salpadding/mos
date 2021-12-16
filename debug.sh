#!/usr/bin/env zsh
nasm -o loader.bin loader.S

# get sectors of loader.bin
LOADER_SIZE=`wc loader.bin | awk '{print $3}'`
LOADER_SECTORS=$(($LOADER_SIZE/512+1))
sed '4s/.*/LOADER_SECTORS equ '$LOADER_SECTORS'/'  boot.inc >boot.inc.replaced
rm boot.inc
mv boot.inc.replaced boot.inc

nasm -o mbr.bin mbr.S
dd if=mbr.bin of=hd60M.img bs=512 count=1 conv=notrunc
dd if=loader.bin of=hd60M.img bs=512 count=$LOADER_SECTORS seek=1 conv=notrunc

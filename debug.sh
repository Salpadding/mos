#!/usr/bin/env zsh
set -e

nasm -o loader.bin loader.S

# get sectors of loader.bin
LOADER_SIZE=`wc loader.bin | awk '{print $3}'`
LOADER_SECTORS=$((($LOADER_SIZE+511)/512))
sed '4s/.*/LOADER_SECTORS equ '$LOADER_SECTORS'/'  boot.inc >boot.inc.replaced
rm boot.inc
mv boot.inc.replaced boot.inc

# build kernel
nasm -o kernel.bin kernel.S
KERNEL_SIZE=`wc kernel.bin | awk '{print $3}'`
KERNEL_SECTORS=$((($KERNEL_SIZE+511)/512))
sed '5s/.*/KERNEL_SECTORS equ '$KERNEL_SECTORS'/'  boot.inc >boot.inc.replaced
rm boot.inc
mv boot.inc.replaced boot.inc


nasm -o mbr.bin mbr.S
dd if=mbr.bin of=hd60M.img bs=512 count=1 conv=notrunc
dd if=loader.bin of=hd60M.img bs=512 count=$LOADER_SECTORS seek=1 conv=notrunc
dd if=kernel.bin of=hd60M.img bs=512 count=$KERNEL_SECTORS seek=$((1+$LOADER_SECTORS)) conv=notrunc

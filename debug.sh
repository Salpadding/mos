#!/usr/bin/env zsh
set -e

# replace $2 line of file $1 with $3
rep_line() {
    # replace space with #
    c=`echo $3 | sed -e s/\ /-/g`
    sed -e $2's/.*/'$c'/' $1 |  sed -e $2's/--*/\ /g' >tmp.txt
    rm $1
    mv tmp.txt $1
}

# sectors of a $1
sectors() {
    sz=`wc $1 | awk '{print $3}'`
    echo $((($sz+511)/512))
}


# select sdl2 as display library on macos
if [[ $OSTYPE == 'darwin'* ]]; then
    rep_line bochsrc.txt 93 'display_library: sdl2'
else
    rep_line bochsrc.txt 93 'display_library: win32, options="gui_debug"'
fi

# build kernel on a 32bit kali system
ssh -t kali@kali 'rm -rf $HOME/kernel' > /dev/null
scp -r kernel kali@kali:/home/kali > /dev/null
ssh -t kali@kali 'pushd $HOME/kernel && gcc -c -O0 -o main.o main.c && ld main.o -Ttext 0xc0001500 -e main -o kernel.bin && rm main.o'
rm -rf kernel.bin
scp kali@kali:/home/kali/kernel/kernel.bin ./kernel.bin

KERNEL_SECTORS=`sectors kernel.bin`
rep_line boot.inc 5 "KERNEL_SECTORS equ $KERNEL_SECTORS"

nasm -o loader.bin loader.S

# get sectors of loader.bin
LOADER_SECTORS=`sectors loader.bin`
rep_line boot.inc 4 "LOADER_SECTORS equ $LOADER_SECTORS"

nasm -o mbr.bin mbr.S
dd if=mbr.bin of=hd60M.img bs=512 count=1 conv=notrunc
dd if=loader.bin of=hd60M.img bs=512 count=$LOADER_SECTORS seek=1 conv=notrunc
dd if=kernel.bin of=hd60M.img bs=512 count=$KERNEL_SECTORS seek=$((1+$LOADER_SECTORS)) conv=notrunc

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

SRC=kernel node make.js

KERNEL_SECTORS=`sectors build/kernel.bin`
rep_line asm/boot.inc 5 "KERNEL_SECTORS equ $KERNEL_SECTORS"

pushd asm>/dev/null
nasm -o ../build/loader.bin loader.S
popd>/dev/null

# get sectors of loader.bin
LOADER_SECTORS=`sectors build/loader.bin`
rep_line asm/boot.inc 4 "LOADER_SECTORS equ $LOADER_SECTORS"

SRC=loader node make.js

pushd asm>/dev/null
nasm -o ../build/mbr.bin mbr.S
popd>/dev/null

dd if=build/mbr.bin of=build/hd60M.img bs=512 count=1 conv=notrunc
dd if=build/loader.bin of=build/hd60M.img bs=512 count=$LOADER_SECTORS seek=1 conv=notrunc
dd if=build/kernel.bin of=build/hd60M.img bs=512 count=$KERNEL_SECTORS seek=$((1+$LOADER_SECTORS)) conv=notrunc

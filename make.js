#!node
const fs = require('fs')
const path = require('path')
const cp = require('child_process')
const memOff = 0x200000

function loader() {
    const bin = fs.readFileSync(path.join(__dirname, 'build/loader.bin'))
    const gdt_off = 8
    const gdt_len = 4
    let gdt = new BigUint64Array(bin.buffer, gdt_off, gdt_len)

    // code segment
    gdt[1]  = gd({
        limit: 0xffffffffn,
        base: 0n,
        rw: true,
        executable: true,
        mode: MODE_PROTECT,
        pri: PRI_KERNEL,
        scale_4k: true,
        system: false
    })

    gdt[2]  = gd({
        limit: 0xffffffffn,
        base: 0n,
        rw: true,
        executable: false,
        mode: MODE_PROTECT,
        pri: PRI_KERNEL,
        scale_4k: true,
        system: false
    })

    fs.writeFileSync('build/loader.bin', bin)
}

const MODE_REAL = 0n
const MODE_PROTECT = 1n
const MODE_LONG = 2n

const PRI_KERNEL = 0n
const PRI_USER = 3n

function gd({limit, base, rw, executable, system, mode, pri, scale_4k }) {
    let lim_low = limit & 0xffffn
    let base_low = base & 0xffffn
    let base_mid = (base & 0xff0000n) >> 16n
    let base_high = (base & 0xff000000n) >> 24n

    let r = 0n
    r = r | lim_low
    r = r | (base_low << 16n)
    r = r | (base_mid << 32n)

    let acc = 1n << 7n
    if (rw)
        acc = acc | (1n << 1n)
    if (executable)
        acc = acc | (1n << 3n)
    if (!system)
        acc = acc | (1n << 4n)

    acc = acc | ((pri & 0x3n) << 5n)

    r = r | (acc << 40n)

    r = r | (((limit & 0xf0000n) >> 16n) << 48n)

    let flags = 0n
    if (scale_4k)
        flags = flags | (1n << 3n)

    switch (mode) {
        case MODE_REAL:
            break
        case MODE_PROTECT:
            flags = flags | (1n << 2n)
            break
        case MODE_LONG:
            flags = flags | (1n << 1n)
    }

    r = r | (flags << 52n)
    r = r | (base_high << 56n)
    return r
}

function kernel() {
    cp.execSync('cargo build --release')

    const bin = fs.readFileSync(path.join(__dirname, 'target/x86-unknown-bare_metal/release/mos'))
    const newBin = Buffer.alloc(bin.length)


    // size of program header = 32 byte
    const phentsize = bin.readUInt16LE(42)
    // offset of program header, 0x34 52
    const phoff = bin.readUInt32LE(28)
    // number of program header
    const phnum = bin.readUInt16LE(44)

    for (let i = 0; i < phnum; i++) {
        const off = phoff + i * phentsize
        const type = bin.readUInt32LE(off)

        if (type !== 1) {
            continue
        }

        // p_offset, segment start of file
        const segOff = bin.readUInt32LE(off + 4)
        const vAddr = bin.readUInt32LE(off + 8)
        const pAddr = bin.readUInt32LE(off + 12)
        const fileSz = bin.readUInt32LE(off + 16)
        const memSz = bin.readUInt32LE(off + 20)


        bin.copy(newBin, vAddr - memOff, segOff, segOff + fileSz)
    }

    fs.writeFileSync('build/kernel.bin', newBin)
}

function preprocess() {
    function id(i)  {
        let x = i.toString(16)
        while (x.length < 2) {
            x = '0' + x
        }
        return '0x'  + x
    }


    const vector_cnt = 33
    const error_vectors = [0x08, 0x0a, 0x0b, 0x0d, 0x0e, 0x11, 0x18, 0x1a, 0x1b, 0x1d, 0x1e]


    const file = fs.readFileSync(path.join(__dirname, 'asm/loader.S'), 'utf8')
    const lines = file.split('\n')

    let j = 0

    for(let i = 0; i < lines.length; i++) {
        if (lines[i].startsWith(';;; IDT_CODE')){
            console.log(`insert code at line ${i}`)
            j = i
        }
    }


    // vector codes
    let idt = '\nint_entries:\n'

    for(let i = 0; i < vector_cnt; i++) {
        idt += `   dd int_${id(i)}_entry\n`
    }

    idt += '\nint_rust:\n   dd 0\n'


    let vcs = '\n'
    for(let i = 0; i < vector_cnt; i++) {
        const err = error_vectors.indexOf(i) >= 0
        vcs += `VECTOR ${id(i)}, ${err ? 'ERROR_CODE' : 'ZERO'}\n`
    }

    let x = lines.slice(0, j).join('\n')

    let y = lines.slice(j, lines.length).join('\n')

    fs.writeFileSync(path.join(__dirname, 'asm/loader.gen.S'), x + idt + vcs + y)

}

switch (process.env.SRC) {
    case 'kernel':
        kernel()
        break
    case 'loader':
        loader()
        break
    case 'gen':
        preprocess()
        break
}
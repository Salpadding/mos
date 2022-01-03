#!node
const fs = require('fs')
const path = require('path')
const cp = require('child_process')
const memOff = 0x100000
const os = require('os')

// resolve path
function rs(s) {
    return path.join(__dirname, s)
}

function writeGDT() {
    const bin = fs.readFileSync(rs('build/loader.bin'))
    const gdt_off = 8
    const gdt_len = 4
    let gdt = new BigUint64Array(bin.buffer, gdt_off, gdt_len)

    // code segment
    gdt[1] = gd({
        limit: 0xffffffffn,
        base: 0n,
        rw: true,
        executable: true,
        mode: MODE_PROTECT,
        pri: PRI_KERNEL,
        scale_4k: true,
        system: false
    })

    gdt[2] = gd({
        limit: 0xffffffffn,
        base: 0n,
        rw: true,
        executable: false,
        mode: MODE_PROTECT,
        pri: PRI_KERNEL,
        scale_4k: true,
        system: false
    })

    console.log(`gdt = ${gdt}`)
    fs.writeFileSync(rs('build/loader.bin'), bin)
}

const MODE_REAL = 0n
const MODE_PROTECT = 1n
const MODE_LONG = 2n

const PRI_KERNEL = 0n
const PRI_USER = 3n

function gd({ limit, base, rw, executable, system, mode, pri, scale_4k }) {
    let g = new BigUint64Array(4)

    let base_high = (base & 0xff000000n) >> 24n
    g[0] = limit & 0xffffn;
    g[1] = base & 0xffffn;
    g[2] |= (base & 0xff0000n) >> 16n

    let acc = 1n << 7n
    if (rw)
        acc = acc | (1n << 1n)
    if (executable)
        acc = acc | (1n << 3n)
    if (!system)
        acc = acc | (1n << 4n)

    acc = acc | ((pri & 0x3n) << 5n)
    g[2] |= acc << 8n
    g[3] |= (limit & 0xf0000n) >> 16n

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

    g[3] |= flags << 4n
    g[3] |= base_high << 8n

    let gn = new Uint16Array(4)

    for(let i = 0; i < 4; i++) {
        gn[i] = Number(g[i])
    }

    return new BigUint64Array(gn.buffer)[0]
}

function buildKernel() {
    const cwd = process.cwd()
    process.chdir(rs('kernel'))
    cp.execSync('cargo build --release')
    process.chdir(__dirname)
    const bin = fs.readFileSync(rs('target/x86-unknown-bare_metal/release/kernel'))
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

    fs.writeFileSync(rs('build/kernel.bin'), newBin)
    process.chdir(cwd)
}

function genLoader() {
    function id(i) {
        let x = i.toString(16)
        while (x.length < 2) {
            x = '0' + x
        }
        return '0x' + x
    }


    const vector_cnt = 33
    const error_vectors = [0x08, 0x0a, 0x0b, 0x0d, 0x0e, 0x11, 0x18, 0x1a, 0x1b, 0x1d, 0x1e]


    const file = fs.readFileSync(rs('asm/loader.S'), 'utf8')
    const lines = file.split('\n')

    let j = 0

    for (let i = 0; i < lines.length; i++) {
        if (lines[i].startsWith(';;; IDT_CODE')) {
            console.log(`insert code at line ${i}`)
            j = i
        }
    }


    // vector codes
    let idt = '\nint_entries:\n'

    for (let i = 0; i < vector_cnt; i++) {
        idt += `   dd int_${id(i)}_entry\n`
    }

    idt += '\nint_rust:\n   dd 0\n'


    let vcs = '\n'
    for (let i = 0; i < vector_cnt; i++) {
        const err = error_vectors.indexOf(i) >= 0
        vcs += `VECTOR ${id(i)}, ${err ? 'ERROR_CODE' : 'ZERO'}\n`
    }

    let x = lines.slice(0, j).join('\n')

    let y = lines.slice(j, lines.length).join('\n')

    fs.writeFileSync(rs('asm/loader.gen.S'), x + idt + vcs + y)
}

function repLine(file, n, s) {
    let content = fs.readFileSync(file, 'utf8')
    let lines = content.split('\n')
    lines[n] = s
    fs.writeFileSync(file, lines.join('\n'))
}

function sectorsOf(f) {
    let st = fs.statSync(f)
    return Math.ceil(st.size / 512)
}

// make build directory if not exists
if (!fs.existsSync(rs('build'))) {
    fs.mkdirSync(rs('build'))
}

// set display library by platform
switch (os.platform()) {
    case 'darwin':
        repLine(rs('bochsrc.txt'), 92, 'display_library: sdl2')
        break
    case 'win32':
        repLine(rs('bochsrc.txt'), 92, 'display_library: win32, options = "gui_debug"')
        break
}

// preprocess loader.S to loader.gen.S
genLoader()

// build kernel
buildKernel()

const kernelSectors = sectorsOf(rs('build/kernel.bin'))

// replace KERNEL SECTORS macro
repLine(
    rs('asm/boot.inc'),
    4, `KERNEL_SECTORS equ ${kernelSectors}`
)

// change directory
process.chdir(rs('asm'))

// build loader to estimate size
cp.execSync('nasm -o ../build/loader.bin loader.gen.S')

const loaderSectors = sectorsOf(rs('build/loader.bin'))

repLine(
    rs('asm/boot.inc'),
    3, `LOADER_SECTORS equ ${loaderSectors}`
)

cp.execSync('nasm -o ../build/loader.bin loader.gen.S')

// write gdt
writeGDT()

cp.execSync('nasm -o ../build/mbr.bin mbr.S')

process.chdir(__dirname)


const cmds = [
    'dd if=build/mbr.bin of=build/disk.img bs=512 count=1 conv=notrunc',
    `dd if=build/loader.bin of=build/disk.img bs=512 count=${loaderSectors} seek=1 conv=notrunc`,
    `dd if=build/kernel.bin of=build/disk.img bs=512 count=${kernelSectors} seek=${1 + loaderSectors} conv=notrunc`
]

for (let c of cmds) {
    cp.execSync(c)
}
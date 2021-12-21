#!node
const fs = require('fs')
const path = require('path')
const cp = require('child_process')
const memOff = 0x300000

cp.execSync('cargo build --release')

const bin = fs.readFileSync(path.join(__dirname, 'target/x86-unknown-bare_metal/release/mos'))
const newBin = Buffer.alloc(bin.length)


// size of program header = 32 byte
const phentsize = bin.readUInt16LE(42)
// offset of program header, 0x34 52
const phoff = bin.readUInt32LE(28)
// number of program header 
const phnum = bin.readUInt16LE(44)

// assert segment load success
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

fs.writeFileSync('kernel.bin', newBin)
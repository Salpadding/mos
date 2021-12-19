const fs = require('fs')
const path = require('path')
const bin = fs.readFileSync(path.join(__dirname, 'kernel.bin'))

// size of program header = 32 byte
const phentsize = bin.readUInt16LE(42)
// offset of program header, 0x34 52
const phoff = bin.readUInt32LE(28)
// number of program header 
const phnum = bin.readUInt16LE(44)

for(let i = 0; i < phnum; i++) {
    const off = phoff + i * phentsize
    const type = bin.readUInt32LE(off)

    if (type === 0) {
        console.log('null pt')
    } else {
        console.log(`pt = 0x${type.toString(16)}`)
    }
}
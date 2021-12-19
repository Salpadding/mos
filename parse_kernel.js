const fs = require('fs')
const path = require('path')
const bin = fs.readFileSync(path.join(__dirname, 'kernel.bin'))

// memory
const mem = fs.readFileSync(path.join(__dirname, 'dump.bin'))


// 1. assert kernel load success
for(let i = 0; i < bin.length; i++) {
    if (mem[i + 0x70000] != bin[i]) {
        console.log('memory load failed')
        process.exit(1)
    }
}


// for(let i = 0x70000 + bin.length; i < mem.length; i++) {
//     if (mem[i] !== 0) {
//         console.log(`write overflow at 0x${i.toString(16)}`)
//         process.exit(1)
//     }
// }

// size of program header = 32 byte
const phentsize = bin.readUInt16LE(42)
// offset of program header, 0x34 52
const phoff = bin.readUInt32LE(28)
// number of program header 
const phnum = bin.readUInt16LE(44)



// assert segment load success
for(let i = 0; i < phnum; i++) {
    const off = phoff + i * phentsize
    const type = bin.readUInt32LE(off)

    if (type === 0) {
        console.log('null pt')
    } else {
        console.log(`pt = 0x${type.toString(16)}`)

        // p_offset, segment start of file 
        const segOff = bin.readUInt32LE(off + 4)
        const vAddr = bin.readUInt32LE(off + 8)
        const pAddr = bin.readUInt32LE(off + 12)
        const fileSz = bin.readUInt32LE(off + 16)
        const memSz = bin.readUInt32LE(off + 20)


        for(let j = 0; j < fileSz; j++) {
            if(mem[vAddr + j] !== bin[segOff + j]) {
                console.log(`segment load failed ${i}`)
                process.exit(1)
            }
        }

        console.log ({
            type: type === 0x6474e551 ? 'PT_GNU_STACK' : type.toString(),
            segOff,
            vAddr: '0x' + vAddr.toString(16),
            pAddr: '0x' + pAddr.toString(16),
            fileSz,
            memSz
        })
    }
}
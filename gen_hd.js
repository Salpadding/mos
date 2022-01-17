const fs = require('fs')
const path = require('path')
const dst = path.join(__dirname, 'build/disk-fs.dmg')
const src = path.join(__dirname, 'partition_table');

const bin = Buffer.alloc(67092480)
const src_bin = fs.readFileSync(src)

src_bin.copy(bin, 0)

fs.writeFileSync(dst, bin)
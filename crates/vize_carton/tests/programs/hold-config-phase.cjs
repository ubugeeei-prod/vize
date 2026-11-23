const fs = require("node:fs");
const paths = JSON.parse(process.argv[2]);
fs.writeFileSync(paths[0], String(process.pid));
process.stderr.write("e".repeat(100000));
const frame = Buffer.from(JSON.stringify(paths));
process.stdout.write(`B${frame.length}\n`);
process.stdout.write(frame);
setInterval(() => {}, 1000);

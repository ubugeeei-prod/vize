require("node:fs").writeFileSync(process.argv[2], String(process.pid));
process.stdout.write('{"value":1}\n');
setInterval(() => {}, 1000);

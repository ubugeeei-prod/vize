const { writeSync } = require("node:fs");
const { stdout, stderr, status, signal } = JSON.parse(process.argv[2]);
writeSync(1, Buffer.from(stdout));
writeSync(2, Buffer.from(stderr));
if (signal) process.kill(process.pid, signal);
else process.exitCode = status;

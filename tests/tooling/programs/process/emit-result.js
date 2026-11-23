import { readFileSync, writeSync } from "node:fs";

const { stdout, stderr, status = 0, delayMs, keepAlive, echoStdin } = JSON.parse(process.argv[2]);
if (echoStdin) writeSync(1, readFileSync(0));
if (stdout) writeSync(1, Buffer.from(stdout));
if (stderr) writeSync(2, Buffer.from(stderr));
process.exitCode = status;
if (keepAlive) setInterval(() => {}, 1000);
else if (delayMs) setTimeout(() => {}, delayMs);

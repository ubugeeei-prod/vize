import { writeSync } from "node:fs";

const [outcome, stdout, stderr] = process.argv.slice(2);
writeSync(1, Buffer.from(stdout, "hex"));
writeSync(2, Buffer.from(stderr, "hex"));
if (outcome === "SIGTERM") process.kill(process.pid, "SIGTERM");
if (outcome === "failure") process.exitCode = 1;

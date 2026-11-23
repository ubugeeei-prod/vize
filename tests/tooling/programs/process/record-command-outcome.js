import fs from "node:fs";

fs.appendFileSync(
  process.argv[2],
  JSON.stringify({ argv: process.argv.slice(2), cwd: process.cwd(), env: process.env }) + "\n",
);
fs.writeSync(1, "original stdout\n");
fs.writeSync(2, "original stderr\n");
const result = process.argv[3];
if (result.startsWith("SIG")) process.kill(process.pid, result);
else process.exit(Number(result));

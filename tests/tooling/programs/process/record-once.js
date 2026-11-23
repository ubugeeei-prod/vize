import { appendFileSync } from "node:fs";

appendFileSync(process.argv[3] ?? "calls", "once");
process.exitCode = Number(process.argv[2]);

import { appendFileSync } from "node:fs";

const [output, code, ...args] = process.argv.slice(2);
const receipt = [process.cwd(), process.env.VIZE_PACKAGE_TASK_PROBE, args];
appendFileSync(output, JSON.stringify(receipt) + "\n");
process.exitCode = Number(code);

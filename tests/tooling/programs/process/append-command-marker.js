import { appendFileSync } from "node:fs";

const [output, marker] = process.argv.slice(2);
appendFileSync(output, marker);

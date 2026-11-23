const { readFileSync } = require("node:fs");

const { source, flags, input } = JSON.parse(readFileSync(0, "utf8"));
const expression = new RegExp(source, flags);
const start = process.hrtime.bigint();
const matched = expression.test(input);
const elapsed = process.hrtime.bigint() - start;
process.stdout.write(JSON.stringify({ m: matched, ms: Number(elapsed) / 1e6 }));

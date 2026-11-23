import assert from "node:assert/strict";
import fs from "node:fs";
assert.deepEqual(process.argv.slice(2), ["--require-prepared"]);
fs.appendFileSync(
  process.env.VIZE_DISCOVERY_RECORDS,
  JSON.stringify({ phase: "prepared" }) + String.fromCharCode(10),
);
process.exit(Number(process.env.VIZE_DISCOVERY_PREPARE_STATUS || 0));

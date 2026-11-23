import assert from "node:assert/strict";
import fs from "node:fs";
import { stampVersion } from "../../../tools/support/release/release/stamp.ts";

const invalid = JSON.parse(process.argv[2]) as string[];
let calls = 0;
for (const name of [
  "opendirSync",
  "statSync",
  "lstatSync",
  "existsSync",
  "accessSync",
  "openSync",
  "readFileSync",
  "writeFileSync",
  "appendFileSync",
  "mkdirSync",
] as const) {
  fs[name] = () => {
    calls++;
    throw new Error("stamp target IO forbidden");
  };
}
for (const version of invalid) assert.equal(await stampVersion("0.1.0", version), false);
assert.equal(calls, 0);
process.stdout.write(JSON.stringify({ cases: invalid.length, targetIoCalls: calls }));

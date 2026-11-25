import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { textFile } from "./files.ts";

const [commit, mode, value] = process.argv.slice(2);
assert.ok(commit && value);
const header = textFile(commit).split("\n\n", 1)[0];
const parent = /^parent ([a-f0-9]{40})$/mu.exec(header)?.[1];
assert.ok(parent, "the retained commit must have a first parent");
if (mode === "expect") assert.equal(parent, value);
else {
  assert.equal(mode, "write");
  writeFileSync(value, parent + "\n");
}

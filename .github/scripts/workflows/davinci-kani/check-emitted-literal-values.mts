import assert from "node:assert/strict";
import fs from "node:fs";
import { runInThisContext } from "node:vm";
const rows = fs.readFileSync(process.argv[2]!, "utf8").trimEnd().split("\n");
for (const row of rows) {
  const [source, literal] = JSON.parse(row) as [string, string];
  assert.equal(JSON.parse(literal), source);
  assert.equal(runInThisContext(literal), source);
}
console.log("Matched", rows.length, "literal values in V8");

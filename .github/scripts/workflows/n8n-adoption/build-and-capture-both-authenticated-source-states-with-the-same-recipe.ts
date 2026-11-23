import assert from "node:assert/strict";
import fs from "node:fs";
const { before, after } = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
assert.equal(before.rustc, after.rustc, "both source phases require the same actual compiler");
assert.equal(before.cargo, after.cargo, "both source phases require the same actual Cargo");
assert.match(after.rustc, /^rustc 1\.100\.0 \(0ef1bc386 2026-11-09\)/u);
assert.notEqual(
  before.binary.sha256,
  after.binary.sha256,
  "distinct production states require distinct producers",
);

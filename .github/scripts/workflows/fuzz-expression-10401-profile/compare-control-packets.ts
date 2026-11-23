import assert from "node:assert/strict";
import { readJson } from "../shared/files.ts";

import { globSync } from "node:fs";
const paths = globSync("evidence/*/controls.json").sort();
assert.equal(paths.length, 2);
const [base, head] = paths.map((file) => readJson<unknown[]>(file));
assert.equal(base!.length, 91);
assert.equal(head!.length, 91);
assert.deepEqual(head, base, "AST, diagnostics or spans changed");
console.log("All 91 complete AST/diagnostic packets match.");

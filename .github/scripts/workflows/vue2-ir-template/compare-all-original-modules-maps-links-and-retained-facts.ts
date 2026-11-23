import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
const folders = readdirSync("evidence");
assert.equal(folders.length, 2);
const read = (role: string) => {
  const folder = folders.find((name) => name.startsWith(`vue2-ir-template-${role}-`));
  assert.ok(folder);
  return JSON.parse(readFileSync(join("evidence", folder, "native.json"), "utf8"));
};
const stable = (row: Record<string, unknown>) => {
  return Object.fromEntries(Object.entries(row).filter(([key]) => key !== "phaseNs"));
};
assert.equal(read("before").rows.length, 19);
assert.deepEqual(read("candidate").rows.map(stable), read("before").rows.map(stable));

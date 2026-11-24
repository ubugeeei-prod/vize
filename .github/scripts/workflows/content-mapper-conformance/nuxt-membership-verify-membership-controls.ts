import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv } from "../../../../tools/ci/lib/scripts/files.ts";

import { readdirSync, statSync } from "node:fs";
const root = requiredEnv("PACKETS");
const expected = {
  "unreferenced-native": 1,
  "unreferenced-stock": 2,
  "referenced-native": 0,
  "referenced-stock": 0,
  "typed-invalid-native": 1,
  "typed-invalid-stock": 2,
  "repair-native": 0,
  "repair-stock": 0,
  "unreferenced-stock-files": 0,
  "referenced-stock-files": 0,
};
const observed = readdirSync(root)
  .filter((name) => name.endsWith(".process.json"))
  .map((name) => name.slice(0, -".process.json".length));
assert.deepEqual(observed.sort(), Object.keys(expected).sort());
for (const [name, status] of Object.entries(expected)) {
  assert.equal(
    readJson<{ exitCode: number }>(join(root, name + ".process.json")).exitCode,
    status,
    name,
  );
  for (const ext of [".stdout", ".stderr"]) assert.ok(statSync(join(root, name + ext)).isFile());
  if (name.endsWith("-native"))
    assert.deepEqual(
      readJson<{ programs: { files: string[] }[] }>(join(root, name + ".stdout")).programs[0]
        ?.files,
      ["nuxt.config.ts", "src/App.vue"],
    );
}

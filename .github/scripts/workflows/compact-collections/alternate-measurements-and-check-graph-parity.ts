import assert from "node:assert/strict";
import fs from "node:fs";
const read = (path: fs.PathOrFileDescriptor) => JSON.parse(fs.readFileSync(path, "utf8"));
for (let i = 1; i <= 5; i += 1) {
  const base = read(`measurements/base-${i}.json`);
  const head = read(`measurements/head-${i}.json`);
  assert.deepEqual(base.cycle_fingerprints, head.cycle_fingerprints);
  assert.deepEqual(base.semantic_fingerprints, head.semantic_fingerprints);
  assert.ok(base.scope_id_bytes === 4 && head.scope_id_bytes === 4);
}

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { fileURLToPath } from "node:url";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("pinned zero-Vue source evidence retains whole authenticated physical inventories", () => {
  const directory = path.join(root, "tests/tooling/fixtures/zero-vue-script-source-evidence");
  const receipts = JSON.parse(fs.readFileSync(path.join(directory, "receipts.json"), "utf8"));
  assert.deepEqual(
    receipts.projects.map((project: any) => project.project),
    ["docsify", "vue-native-core", "wakapi", "petite-vue"],
  );
  for (const receipt of receipts.projects) {
    const bytes = gunzipSync(fs.readFileSync(path.join(directory, `${receipt.project}.json.gz`)));
    assert.equal(bytes.length, receipt.manifestBytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), receipt.manifestSha256);
    const manifest = JSON.parse(bytes.toString("utf8"));
    assert.equal(manifest.revision, receipt.revision);
    const counts: Record<string, number> = {};
    for (const file of manifest.files)
      counts[path.extname(file.file)] = (counts[path.extname(file.file)] ?? 0) + 1;
    assert.deepEqual(counts, receipt.sourceCounts);
  }
});

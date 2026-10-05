import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lsp/zed-workspace-profile");
const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "case.json"), "utf8"));

test("Zed config regression retains the complete original config and authored SFC bytes", () => {
  for (const [name, digest] of Object.entries({
    ...manifest.originalSha256,
    ...manifest.controlSha256,
  })) {
    assert.equal(
      createHash("sha256").update(fs.readFileSync(path.join(fixture, name))).digest("hex"),
      digest,
    );
  }
  assert.equal(manifest.issue, 8007);
  assert.equal(manifest.relatedIssue, 7196);
  assert.equal(manifest.cases.length, 6);
  assert.equal(new Set(manifest.cases.map((row: { id: string }) => row.id)).size, 6);
});

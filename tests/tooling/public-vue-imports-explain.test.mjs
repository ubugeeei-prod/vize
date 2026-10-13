import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256 } from "./lint-default-correctness-reference.mjs";
import {
  composeInheritanceDocument,
  decodeInheritanceBody,
  loadInheritanceDocumentation,
} from "./support/attribute-inheritance-documentation-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const inheritance = loadInheritanceDocumentation(root);
await test("private import example changes only its three whole explain snapshots", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/public-vue-imports-8090");
  const primary = JSON.parse(fs.readFileSync(path.join(fixture, "primary-sources.json")));
  const bytes = fs.readFileSync(path.join(fixture, primary.explainSnapshot.path));
  assert.equal(sha256(bytes), primary.explainSnapshot.sha256);
  const contract = JSON.parse(gunzipSync(bytes));
  assert.equal(contract.schema, "vize.lint.public-vue-imports-explain-contract");
  assert.equal(contract.snapshots.length, 3);
  const source = fs.readFileSync(path.join(root, primary.sourceProducer.path), "utf8");
  assert.equal(sha256(source), contract.producerSha256);
  const valid = source
    .split("//! ### Valid\n//! ```ts\n")[1]
    .split("//! ```")[0]
    .split("\n")
    .filter((line) => line.startsWith("//! "))
    .map((line) => line.slice(4));
  assert.deepEqual(valid, [
    "import { ref, h } from 'vue'",
    "import { isString } from '@vue/shared'",
  ]);
  for (const row of contract.snapshots) {
    const before = Buffer.from(row.beforeBase64, "base64");
    assert.equal(sha256(before), row.beforeSha256);
    assert.equal(before.length, row.beforeBytes);
    const marker = "  import { ref, h } from 'vue'\n=== script/prefer-ref-over-reactive";
    assert.equal(before.toString("utf8").split(marker).length, 2);
    const expected = before
      .toString("utf8")
      .replace(
        marker,
        valid.map((line) => `  ${line}`).join("\n") + "\n=== script/prefer-ref-over-reactive",
      );
    // Retain the complete original private-import successor before composing
    // the separately named, whole nested-attribute-inheritance successor.
    assert.equal(sha256(expected), row.afterSha256);
    assert.equal(Buffer.byteLength(expected), row.afterBytes);
    const inherited = inheritance.documents.find((item) => item.path === row.path);
    assert(inherited);
    assert.equal(decodeInheritanceBody(inherited.before).toString("utf8"), expected);
    const after = fs.readFileSync(path.join(root, row.path));
    assert.equal(after.toString("utf8"), composeInheritanceDocument(expected, row.path));
  }
});

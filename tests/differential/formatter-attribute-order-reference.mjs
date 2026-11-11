// Closed current references preserve the historical formatter manifest and outputs.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { assertClosedFormatterManifest } from "./formatter-sole-child-width-reference.mjs";

const AUTHORITY =
  "tests/_fixtures/differential/formatter-regressions/attribute-order-12826/current-references.json";
const AUTHORITY_SHA256 = "b41903aadbe3c97c69a2ff1ba33c9ccec6c0ccec422ddff8bb2f7315c414d0f2";

function containedBytes(root, relative) {
  const file = fs.realpathSync(path.join(root, relative));
  const contained = path.relative(fs.realpathSync(root), file);
  assert(contained && !contained.startsWith("..") && !path.isAbsolute(contained));
  return fs.readFileSync(file);
}

export function attributeOrderReference(root, fixture) {
  const authorityBytes = containedBytes(root, AUTHORITY);
  assert.equal(sha256(authorityBytes), AUTHORITY_SHA256, "attribute-order authority changed");
  const authority = JSON.parse(authorityBytes);
  assert.equal(authority.schema, "vize.formatter-attribute-order-current-references");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 12826);
  const row = authority.records.find((record) => record.id === fixture.id);
  if (!row) return {};
  assertClosedFormatterManifest(root);
  assert.equal(sha256(fixture.input), row.inputSha256, "whole original input changed");
  assert.equal(
    sha256(fixture.expected),
    row.historicalExpectedSha256,
    "historical formatter output changed",
  );
  assert.deepEqual(fixture.argv, ["fmt", "--no-config", "--write", "App.vue"]);
  assert.equal(fixture.config.length, 0, "default formatter configuration changed");
  const currentExpected = containedBytes(root, row.currentPath);
  assert.equal(sha256(currentExpected), row.currentExpectedSha256, "current output changed");
  assert(!currentExpected.equals(fixture.expected), "historical mismatch remains observable");
  return {
    currentExpected,
    currentReference: {
      issue: authority.issue,
      authority: { path: AUTHORITY, sha256: AUTHORITY_SHA256 },
      inputSha256: row.inputSha256,
      historicalExpectedSha256: row.historicalExpectedSha256,
      currentExpectedSha256: row.currentExpectedSha256,
      justification: authority.justification,
    },
  };
}

import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { exposeReadbacks } from "../../tools/support/levels/expose-profile-readbacks.ts";

const source = fs.readFileSync(
  new URL("../../davinci/vize_l0/src/profiler/snapshot.rs", import.meta.url),
  "utf8",
);

test("the host consumes the same owned readbacks without another collector stage", () => {
  assert.equal(exposeReadbacks(source), source);
  const old = source
    .replace("pub fn span_snapshot", "pub(super) fn span_snapshot")
    .replace("pub fn counter_snapshot", "pub(super) fn counter_snapshot");
  assert.equal(exposeReadbacks(old), source);
  assert.equal((source.match(/metrics\.clone\(\)/gu) ?? []).length, 3);
  assert.equal((source.match(/counter\.clone\(\)/gu) ?? []).length, 1);
  assert.doesNotMatch(source, /serde|vize_carton|ProfileExport/u);
});

test("partial or duplicate readback exposure rejects before a prospective write", () => {
  assert.throws(
    () => exposeReadbacks(source.replace("pub fn span_snapshot", "pub(super) fn span_snapshot")),
    /partial/u,
  );
  assert.throws(() => exposeReadbacks(source + "\npub fn span_snapshot(&self)"), /ambiguous/u);
  assert.throws(
    () => exposeReadbacks(source.replace("pub fn counter_snapshot", "pub fn unexpected_snapshot")),
    /ambiguous/u,
  );
});

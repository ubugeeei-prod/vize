import assert from "node:assert/strict";
import { test } from "node:test";
import { rustMetadataString } from "../../docs/scripts/rules/rust-metadata.ts";

void test("rule documentation retains literal and constant-backed metadata identities", () => {
  for (const declaration of [
    'const RULE: &str = "vue/comment-directive";',
    'pub const RULE: &\'static str =\n "vue/comment-directive";',
  ]) {
    assert.equal(rustMetadataString("name: RULE,", "name", declaration), "vue/comment-directive");
  }
  assert.equal(rustMetadataString('name: "vue/html-end-tags",', "name", ""), "vue/html-end-tags");
  assert.equal(
    rustMetadataString('description: "An escaped \\"quote\\".",', "description", ""),
    'An escaped "quote".',
  );
});

void test("missing constants fail while reused metadata fields do not create a second rule", () => {
  assert.throws(() => rustMetadataString("name: MISSING,", "name", ""), {
    message: "Unresolved metadata string: name: MISSING",
  });
  assert.equal(rustMetadataString("name: META.name,", "name", ""), null);
  assert.equal(rustMetadataString('other_name: "foreign/id",', "name", ""), null);
});

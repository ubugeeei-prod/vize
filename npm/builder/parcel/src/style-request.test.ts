import assert from "node:assert/strict";
import { test } from "node:test";
import { styleRequest } from "./style-request.ts";

void test("external style queries preserve question marks, Unicode, spaces, and fragments", () => {
  const value = styleRequest(
    "./é style.css?theme=one?two&empty=#sheet",
    "data-v-123",
    true,
    "src/😺.vue",
  );
  const parsed = new URL(value, "file:///project/");
  assert.ok(value.startsWith("./%C3%A9%20style.css?"));
  assert.equal(decodeURIComponent(parsed.pathname), "/project/é style.css");
  assert.equal(parsed.searchParams.get("theme"), "one?two");
  assert.equal(parsed.searchParams.get("empty"), "");
  assert.equal(parsed.searchParams.get("vize-scope"), "data-v-123");
  assert.equal(parsed.searchParams.get("vize-vars"), "data-v-123");
  assert.equal(parsed.searchParams.get("vize-owner"), "src/😺.vue");
  assert.equal(parsed.hash, "#sheet");
});

void test("unscoped development imports only carry CSS-variable context", () => {
  const parsed = new URL(styleRequest("./style.css", "data-v-abc", false), "file:///project/");
  assert.deepEqual([...parsed.searchParams], [["vize-vars", "data-v-abc"]]);
});

import assert from "node:assert/strict";
import { test } from "node:test";
import { ref, shallowRef } from "vue";

import { refDefault } from "./ref-default.ts";

void test("reads the default while the source is nullish and writes through", () => {
  const raw = ref<string | null | undefined>();
  const fallback = shallowRef("Anonymous");
  const name = refDefault(raw, fallback);

  assert.equal(name.value, "Anonymous");
  fallback.value = "Guest";
  assert.equal(name.value, "Guest");
  name.value = "Ada";
  assert.equal(raw.value, "Ada");
  assert.equal(name.value, "Ada");
  name.value = null;
  assert.equal(raw.value, null);
  assert.equal(name.value, "Guest");
});

void test("keeps falsy non-nullish values", () => {
  const raw = ref<number | undefined>(0);
  assert.equal(refDefault(raw, 10).value, 0);
});

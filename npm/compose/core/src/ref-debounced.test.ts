import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { refDebounced } from "./ref-debounced.ts";
import { FakeTimeouts } from "./testing/fake-timeouts.ts";

void test("settles after the quiet period", () => {
  const timers = new FakeTimeouts();
  const query = shallowRef("");
  const debounced = refDebounced(query, 300, { runOnServer: true, scheduler: timers });
  query.value = "v";
  query.value = "vize";
  assert.equal(debounced.value, "");
  timers.advance(300);
  assert.equal(debounced.value, "vize");
});

void test("mirrors synchronously on the server", () => {
  const query = shallowRef("a");
  const debounced = refDebounced(query, 300);
  query.value = "b";
  assert.equal(debounced.value, "b");
});

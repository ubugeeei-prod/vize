import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { usePageLeave } from "./page-leave.ts";
import { eventWith } from "./testing/fake-dom.ts";

void test("flags pointer exits without a related target", () => {
  const host = new EventTarget();
  const scope = effectScope();
  const left = scope.run(() => usePageLeave({ host }));
  assert.equal(left?.value, false);

  host.dispatchEvent(eventWith("mouseout", { relatedTarget: {} }));
  assert.equal(left?.value, false);
  host.dispatchEvent(eventWith("mouseout", { relatedTarget: null }));
  assert.equal(left?.value, true);
  host.dispatchEvent(new Event("mouseover"));
  assert.equal(left?.value, false);

  scope.stop();
  host.dispatchEvent(eventWith("mouseout", { relatedTarget: null }));
  assert.equal(left?.value, false);
});

void test("server renders report false", () => {
  assert.equal(usePageLeave({ host: null }).value, false);
});

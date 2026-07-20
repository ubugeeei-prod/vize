import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

void test("reports that cleanup cannot be registered outside a scope", () => {
  let cleanupCalls = 0;

  assert.equal(
    tryOnScopeDispose(() => (cleanupCalls += 1)),
    false,
  );
  assert.equal(cleanupCalls, 0);
});

void test("runs registered cleanup exactly once when the scope stops", () => {
  const scope = effectScope();
  let cleanupCalls = 0;
  const registered = scope.run(() => tryOnScopeDispose(() => (cleanupCalls += 1)));

  assert.equal(registered, true);
  assert.equal(cleanupCalls, 0);
  scope.stop();
  scope.stop();
  assert.equal(cleanupCalls, 1);
});

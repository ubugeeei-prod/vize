import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, onScopeDispose, shallowRef } from "vue";

import { createSharedComposable } from "./create-shared-composable.ts";

void test("shares one instance per set of live subscribers", () => {
  let created = 0;
  let disposed = 0;
  const useShared = createSharedComposable(
    (start: number) => {
      created += 1;
      onScopeDispose(() => {
        disposed += 1;
      });
      return shallowRef(start);
    },
    { shareOnServer: true },
  );

  const first = effectScope();
  const second = effectScope();
  const a = first.run(() => useShared(1));
  const b = second.run(() => useShared(99));
  assert.equal(a, b);
  assert.equal(a?.value, 1);
  first.stop();
  assert.equal(disposed, 0);
  second.stop();
  assert.equal(disposed, 1);

  const third = effectScope();
  const c = third.run(() => useShared(7));
  assert.equal(c?.value, 7);
  assert.equal(created, 2);
  third.stop();
});

void test("does not share on the server by default", () => {
  const useShared = createSharedComposable(() => ({}));
  assert.notEqual(useShared(), useShared());
});

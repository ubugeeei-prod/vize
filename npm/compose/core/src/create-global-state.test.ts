import assert from "node:assert/strict";
import { test } from "node:test";
import { computed, effectScope, shallowRef } from "vue";

import { createGlobalState } from "./create-global-state.ts";

void test("creates the state once, lazily, and outlives caller scopes", () => {
  let runs = 0;
  const useState = createGlobalState(() => {
    runs += 1;
    const count = shallowRef(1);
    return { count, double: computed(() => count.value * 2) };
  });
  assert.equal(runs, 0);

  const scope = effectScope();
  const first = scope.run(() => useState());
  scope.stop();
  const second = useState();
  assert.equal(first, second);
  assert.equal(runs, 1);
  second.count.value = 4;
  assert.equal(second.double.value, 8);
});

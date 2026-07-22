import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { usePrevious } from "./use-previous.ts";

void test("starts undefined and shifts on every synchronous write", () => {
  const source = shallowRef(1);
  const previous = usePrevious(source);

  assert.equal(previous.value, undefined);
  source.value = 2;
  assert.equal(previous.value, 1);
  source.value = 3;
  assert.equal(previous.value, 2);

  // Two writes in the same tick are both observed instead of collapsing.
  source.value = 4;
  source.value = 5;
  assert.equal(previous.value, 4);
});

void test("reports the provided initial value until the first change", () => {
  const source = shallowRef("/settings");
  const previous = usePrevious(source, "/");

  assert.equal(previous.value, "/");
  source.value = "/profile";
  assert.equal(previous.value, "/settings");
});

void test("treats an explicit undefined initial argument as an initial value", () => {
  const source = shallowRef<string | undefined>("first");
  const previous = usePrevious(source, undefined);

  assert.equal(previous.value, undefined);
  source.value = "second";
  assert.equal(previous.value, "first");
});

void test("ignores writes that do not change the value", () => {
  const source = shallowRef("stable");
  const previous = usePrevious(source, "initial");

  source.value = "stable";
  assert.equal(previous.value, "initial");

  const items: readonly number[] = [1];
  const objectSource = shallowRef(items);
  const previousItems = usePrevious(objectSource);
  objectSource.value = items;
  assert.equal(previousItems.value, undefined);
});

void test("tracks getter sources", () => {
  const base = shallowRef(10);
  const previous = usePrevious(() => base.value * 2);

  base.value = 20;
  assert.equal(previous.value, 20);
  base.value = 30;
  assert.equal(previous.value, 40);
});

void test("stops tracking when the owning scope stops", () => {
  const source = shallowRef("a");
  const scope = effectScope();
  const previous = scope.run(() => usePrevious(source));
  assert.ok(previous);

  source.value = "b";
  assert.equal(previous.value, "a");

  scope.stop();
  source.value = "c";
  assert.equal(previous.value, "a");
});

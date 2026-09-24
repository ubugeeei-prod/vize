import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref, shallowRef } from "vue";

import { FakeClock } from "./testing/fake-clock.ts";
import { until } from "./until.ts";

void test("resolves immediately when the condition already holds", async () => {
  const source = shallowRef<string | null>("ready");
  assert.equal(await until(source).toBe("ready"), "ready");
  assert.equal(await until(source).not.toBeNull(), "ready");
  assert.equal(await until(source).toBeTruthy(), "ready");
});

void test("waits for later changes", async () => {
  const source = shallowRef<number | null>(null);
  const pending = until(source).toMatch((value): value is number => value !== null && value > 2);
  source.value = 1;
  source.value = 3;
  assert.equal(await pending, 3);

  const value = shallowRef(1);
  const target = shallowRef(5);
  const toTarget = until(value).toBe(target);
  target.value = 2;
  value.value = 2;
  assert.equal(await toTarget, 2);
});

void test("covers null, undefined, NaN, containment, and negations", async () => {
  const source = shallowRef<number | null | undefined>(1);
  const toNull = until(source).toBeNull();
  source.value = null;
  assert.equal(await toNull, null);
  const toUndefined = until(source).toBeUndefined();
  source.value = undefined;
  assert.equal(await toUndefined, undefined);
  const toNaN = until(source).toBeNaN();
  source.value = Number.NaN;
  assert.ok(Number.isNaN(await toNaN));
  const notUndefined = until(source).not.toBeUndefined();
  assert.ok(Number.isNaN(await notUndefined));

  const list = shallowRef<readonly string[]>([]);
  const contains = until(list).toContain("x");
  list.value = ["a", "x"];
  assert.deepEqual(await contains, ["a", "x"]);

  const text = shallowRef("abc");
  const containsText = until(text).toContain("zz");
  text.value = "azzb";
  assert.equal(await containsText, "azzb");

  const flag = shallowRef(1);
  const falsy = until(flag).not.toBeTruthy();
  const notOne = until(flag).not.toBe(1);
  const notPositive = until(flag).not.toMatch((value) => value > 0);
  flag.value = 0;
  assert.equal(await falsy, 0);
  assert.equal(await notOne, 0);
  assert.equal(await notPositive, 0);
});

void test("changed and changedTimes ignore the current value", async () => {
  const source = shallowRef(0);
  const once = until(source).changed();
  const twice = until(source).changedTimes(2);
  source.value = 1;
  assert.equal(await once, 1);
  source.value = 2;
  assert.equal(await twice, 2);
  await assert.rejects(until(source).changedTimes(0), RangeError);
});

void test("timeouts reject with a tagged error and release the watcher", async () => {
  const clock = new FakeClock();
  const source = shallowRef(0);
  const pending = until(source).toBe(5, { timeout: 100, scheduler: clock.timeout });
  clock.advance(100);
  await assert.rejects(pending, (error: unknown) => {
    assert.ok(error instanceof Error);
    assert.equal((error as Error & { code?: string }).code, "VIZE_COMPOSE_UNTIL_TIMEOUT");
    return true;
  });
  source.value = 5;
  assert.equal(clock.size, 0);
  await assert.rejects(until(source).toBe(1, { timeout: -1 }), RangeError);
});

void test("abort signals reject with their reason", async () => {
  const source = shallowRef(0);
  const controller = new AbortController();
  const pending = until(source).toBe(1, { signal: controller.signal });
  controller.abort("cancelled");
  await assert.rejects(pending, (reason: unknown) => reason === "cancelled");
  await assert.rejects(
    until(source).toBe(1, { signal: controller.signal }),
    (reason: unknown) => reason === "cancelled",
  );
});

void test("supports batched flush and deep watching", async () => {
  const source = ref({ items: [] as number[] });
  const pending = until(source).toMatch((value) => value.items.length > 0, {
    flush: "pre",
    deep: true,
  });
  source.value.items.push(1);
  await nextTick();
  assert.deepEqual((await pending).items, [1]);
});

void test("works inside a scope that is later stopped", async () => {
  const source = shallowRef(0);
  const scope = effectScope();
  const pending = scope.run(() => until(source).toBe(1));
  source.value = 1;
  scope.stop();
  assert.equal(await pending, 1);
});

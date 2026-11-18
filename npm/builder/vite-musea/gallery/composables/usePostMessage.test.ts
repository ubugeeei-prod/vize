import assert from "node:assert/strict";
import test from "node:test";

import { reactive } from "vue";

import { ownRecordValue, snapshotMessageRecord } from "./usePostMessage.ts";

void test("props message snapshots unwrap Vue proxies and retain own prototype names", () => {
  const attrs = reactive(
    JSON.parse(
      '{"empty":{},"nested":[{"label":"Edited 🎨","__proto__":{"retained":true}}],"__proto__":{"nested":true},"constructor":"Authored constructor","hasOwnProperty":"Authored method name"}',
    ),
  );
  const payload = snapshotMessageRecord(attrs);

  assert.deepEqual(
    payload,
    JSON.parse(
      '{"empty":{},"nested":[{"label":"Edited 🎨","__proto__":{"retained":true}}],"__proto__":{"nested":true},"constructor":"Authored constructor","hasOwnProperty":"Authored method name"}',
    ),
  );
  assert.doesNotThrow(() => structuredClone(payload));
});

void test("props message snapshots read own values without proxy getter instrumentation", () => {
  const source = JSON.parse('{"label":"Authored label","hasOwnProperty":"Authored method name"}');
  const instrumented = new Proxy(source, {
    get(target, key, receiver) {
      if (key === "hasOwnProperty") return () => false;
      return Reflect.get(target, key, receiver);
    },
  });
  const payload = snapshotMessageRecord(instrumented);

  assert.deepEqual(payload, source);
  assert.equal(Reflect.get(payload, "hasOwnProperty"), "Authored method name");
  assert.doesNotThrow(() => structuredClone(payload));
});

void test("props message snapshots recursively unwrap nested Vue proxies", () => {
  const nested = reactive({ items: [reactive({ label: "Nested" })] });
  const payload = snapshotMessageRecord({ nested });

  assert.deepEqual(payload, { nested: { items: [{ label: "Nested" }] } });
  assert.doesNotThrow(() => structuredClone(payload));
});

void test("props message snapshots omit non-cloneable JSON values", () => {
  const payload = snapshotMessageRecord({
    callback: () => "ignored",
    nested: { symbol: Symbol("ignored"), label: "Retained" },
  });

  assert.deepEqual(payload, { nested: { label: "Retained" } });
  assert.doesNotThrow(() => structuredClone(payload));
});

void test("own record reads retain Vue-instrumented prototype names", () => {
  const values = reactive(
    JSON.parse('{"constructor":"Authored constructor","hasOwnProperty":"Authored method"}'),
  );

  assert.equal(ownRecordValue(values, "constructor"), "Authored constructor");
  assert.equal(ownRecordValue(values, "hasOwnProperty"), "Authored method");
});

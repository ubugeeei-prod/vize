import assert from "node:assert/strict";
import test from "node:test";

import { reactive } from "vue";

import { snapshotMessageRecord } from "./usePostMessage.ts";

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

import assert from "node:assert/strict";
import { test } from "node:test";
import { nativeOperation } from "./trace.ts";

void test("disabled profiling executes the operation exactly once without recording", () => {
  let calls = 0;
  const result = {};
  assert.equal(
    nativeOperation(
      { enabled: false, createMeasurement: () => assert.fail("disabled trace") },
      "compileSfc",
      "组件.vue",
      () => {
        calls++;
        return result;
      },
    ),
    result,
  );
  assert.equal(calls, 1);
});

void test("native failures retain exact identity and close their trace once", () => {
  const failure = new Error("native refusal");
  let ends = 0;
  assert.throws(
    () =>
      nativeOperation(
        {
          enabled: true,
          createMeasurement: (name, category, filename) => {
            assert.equal(name, "napi:compileSfc");
            assert.equal(category, "native");
            assert.equal(filename, "组件.vue");
            return {
              end: () => {
                ends++;
              },
            };
          },
        },
        "compileSfc",
        "组件.vue",
        () => {
          throw failure;
        },
      ),
    (error) => error === failure,
  );
  assert.equal(ends, 1);
});

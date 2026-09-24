import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick } from "vue";

import type { StandardSchemaV1 } from "./standard-schema.ts";
import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useField } from "./use-field.ts";

async function flush(): Promise<void> {
  await nextTick();
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
}

const minLength = (length: number) => (value: string) =>
  value.length >= length ? undefined : `At least ${String(length)} characters`;

void test("validates on change with plain rules and tracks dirty state", async () => {
  const field = useField("", { name: "user", rules: [minLength(3)] });
  assert.equal(field.name, "user");
  assert.equal(field.valid.value, true, "nothing validates before the first change");

  field.value.value = "ab";
  await flush();
  assert.equal(field.error.value, "At least 3 characters");
  assert.equal(field.dirty.value, true);

  field.value.value = "abc";
  await flush();
  assert.equal(field.valid.value, true);
});

void test("accepts Standard Schema rules alongside plain ones", async () => {
  const positive: StandardSchemaV1<number, number> = {
    "~standard": {
      version: 1,
      vendor: "test",
      validate: (value) =>
        typeof value === "number" && value > 0
          ? { value }
          : { issues: [{ message: "Must be positive" }] },
    },
  };
  const field = useField(0, {
    rules: [positive, (value) => (value % 2 === 0 ? undefined : "Must be even")],
    validateOn: "manual",
  });

  field.value.value = -3;
  await flush();
  assert.deepEqual(field.errors.value, [], "manual fields do not validate on change");
  assert.equal(await field.validate(), false);
  assert.deepEqual(field.errors.value, ["Must be positive", "Must be even"]);
  field.value.value = 4;
  assert.equal(await field.validate(), true);
});

void test("validates on blur when configured", async () => {
  const field = useField("", { rules: minLength(1), validateOn: "blur" });
  field.value.value = "";
  await flush();
  assert.equal(field.valid.value, true);
  field.onBlur();
  await flush();
  assert.equal(field.touched.value, true);
  assert.equal(field.error.value, "At least 1 characters");
});

void test("discards superseded async validations", async () => {
  const signals: AbortSignal[] = [];
  const resolvers: ((message: string | undefined) => void)[] = [];
  const field = useField("", {
    validateOn: "manual",
    rules: (_value, { signal }) =>
      new Promise<string | undefined>((resolve) => {
        signals.push(signal);
        resolvers.push(resolve);
      }),
  });
  const first = field.validate();
  const second = field.validate();
  assert.equal(signals[0]?.aborted, true);
  assert.equal(field.validating.value, true);
  resolvers[1]?.(undefined);
  resolvers[0]?.("stale");
  assert.equal(await first, false);
  assert.equal(await second, true);
  assert.deepEqual(field.errors.value, []);
  assert.equal(field.validating.value, false);
});

void test("resets without triggering validation and supports server errors", async () => {
  let calls = 0;
  const field = useField(() => ({ tags: ["a"] }), {
    rules: () => {
      calls += 1;
      return undefined;
    },
  });
  field.value.value.tags.push("b");
  await flush();
  assert.equal(calls, 1, "deep changes validate");

  field.setErrors(["Taken on the server"]);
  assert.equal(field.error.value, "Taken on the server");
  field.reset();
  await flush();
  assert.equal(calls, 1, "reset does not validate");
  assert.deepEqual(field.value.value, { tags: ["a"] });
  assert.equal(field.valid.value, true);
  assert.equal(field.dirty.value, false);

  field.reset({ tags: [] });
  assert.equal(field.dirty.value, false);
});

void test("aborts pending validation when the scope stops", () => {
  let signal: AbortSignal | undefined;
  const scope = effectScope();
  const field = scope.run(() =>
    useField("", {
      validateOn: "manual",
      rules: (_value, context) => {
        signal = context.signal;
        return new Promise<undefined>(() => undefined);
      },
    }),
  );
  assert.ok(field);
  void field.validate();
  scope.stop();
  assert.equal(signal?.aborted, true);
});

void test("server rendering exposes the initial state without validating", async () => {
  let calls = 0;
  const state = await renderComposableOnServer(() => {
    const field = useField("x", {
      rules: () => {
        calls += 1;
        return "invalid";
      },
    });
    return { value: field.value, valid: field.valid, dirty: field.dirty };
  });
  assert.equal(state, '{"value":"x","valid":true,"dirty":false}');
  assert.equal(calls, 0);
});

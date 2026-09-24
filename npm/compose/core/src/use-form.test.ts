import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick } from "vue";

import type { StandardSchemaResult, StandardSchemaV1 } from "./standard-schema.ts";
import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useForm } from "./use-form.ts";

interface Profile {
  name: string;
  age: number;
  address: { city: string; zip: string };
  tags: string[];
}

const initialProfile = (): Profile => ({
  name: "",
  age: 20,
  address: { city: "Tokyo", zip: "100" },
  tags: ["a", "b", "c"],
});

function schema<Output>(
  validate: (
    value: unknown,
  ) => StandardSchemaResult<Output> | Promise<StandardSchemaResult<Output>>,
): StandardSchemaV1<Profile, Output> {
  return { "~standard": { version: 1, vendor: "test", validate } };
}

async function flush(): Promise<void> {
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
}

void test("binds typed fields to nested values and tracks dirty and touched state", async () => {
  const form = useForm({ initialValues: initialProfile });
  const city = form.field("address.city");

  assert.equal(city.value.value, "Tokyo");
  assert.equal(city.path, "address.city");
  assert.equal(city.dirty.value, false);
  city.value.value = "Osaka";
  assert.equal(form.values.value.address.city, "Osaka");
  assert.equal(city.dirty.value, true);
  assert.equal(form.dirty.value, true);

  city.onBlur();
  assert.equal(city.touched.value, true);
  assert.equal(form.isTouched("address.city"), true);

  city.value.value = "Tokyo";
  assert.equal(form.dirty.value, false, "dirty compares against initial values");

  form.values.value.name = "direct";
  await nextTick();
  assert.equal(form.field("name").dirty.value, true, "direct mutation is tracked");
});

void test("runs field, form, and schema validators and merges their errors", async () => {
  const form = useForm({
    initialValues: initialProfile,
    validators: {
      name: (name) => (name === "" ? "Name is required" : undefined),
      "address.zip": async (zip) => (/^\d{3}-\d{4}$/.test(zip) ? undefined : ["Bad zip"]),
    },
    validate: (values) => (values.age < 18 ? { age: ["Too young"] } : undefined),
    schema: schema((value) =>
      typeof value === "object" && value !== null && Reflect.get(value, "name") === "admin"
        ? { issues: [{ message: "Reserved", path: [{ key: "name" }] }] }
        : { value: { ok: true as const } },
    ),
  });

  const invalid = await form.validate();
  assert.equal(invalid.status, "invalid");
  assert.deepEqual(form.errors.value, { name: ["Name is required"], "address.zip": ["Bad zip"] });
  assert.equal(form.valid.value, false);
  assert.equal(form.field("name").error.value, "Name is required");

  form.setValue("name", "admin");
  form.setValue("address.zip", "100-0001");
  form.setValue("age", 10);
  const stillInvalid = await form.validate();
  assert.equal(stillInvalid.status, "invalid");
  assert.deepEqual(form.errors.value, { name: ["Reserved"], age: ["Too young"] });

  form.setValue("name", "Ada");
  form.setValue("age", 30);
  const valid = await form.validate();
  assert.deepEqual(valid, { status: "valid", value: { ok: true } }, "schema output is returned");
  assert.equal(form.valid.value, true);
});

void test("supersedes stale async validations and aborts their signal", async () => {
  const signals: AbortSignal[] = [];
  const resolvers: (() => void)[] = [];
  const form = useForm({
    initialValues: initialProfile,
    validators: {
      name: (_name, { signal }) =>
        new Promise<string>((resolve) => {
          signals.push(signal);
          resolvers.push(() => resolve("stale"));
        }),
    },
  });

  const first = form.validate();
  const second = form.validate();
  assert.equal(signals[0]?.aborted, true);
  assert.equal(form.validating.value, true);
  resolvers[1]?.();
  resolvers[0]?.();
  assert.deepEqual(await first, { status: "superseded" });
  assert.equal((await second).status, "invalid");
  assert.equal(form.validating.value, false);
});

void test("validates on blur or change according to the triggers", async () => {
  let calls = 0;
  const form = useForm({
    initialValues: initialProfile,
    validateOn: "blur",
    revalidateOn: "change",
    validators: {
      name: (name) => {
        calls += 1;
        return name.length < 2 ? "Too short" : undefined;
      },
    },
  });
  const name = form.field("name");

  name.value.value = "x";
  await flush();
  assert.equal(calls, 0, "no change validation before the first submit");
  name.onBlur();
  await flush();
  assert.equal(name.error.value, "Too short");

  await form.submit();
  const before = calls;
  name.value.value = "xyz";
  await flush();
  assert.equal(calls, before + 1, "change re-validation after submit");
  assert.equal(name.error.value, undefined);
});

void test("submits schema output only when valid and marks invalid fields touched", async () => {
  const submitted: unknown[] = [];
  const form = useForm({
    initialValues: initialProfile,
    schema: schema(async (value) => ({ value: { parsed: value } })),
    validators: { name: (name) => (name ? undefined : "Required") },
    onSubmit: (value) => {
      submitted.push(value);
    },
  });

  const invalid = await form.submit();
  assert.equal(invalid.status, "invalid");
  assert.equal(submitted.length, 0);
  assert.equal(form.isTouched("name"), true);
  assert.equal(form.submitCount.value, 1);

  form.setValue("name", "Ada");
  let prevented = false;
  const event = new Event("submit", { cancelable: true });
  event.preventDefault = () => {
    prevented = true;
  };
  const listener = form.handleSubmit();
  const result = await listener(event);
  assert.equal(prevented, true);
  assert.equal(result.status, "valid");
  assert.equal(submitted.length, 1);
  assert.equal(form.submitting.value, false);
});

void test("field arrays keep stable keys and carry errors through moves", async () => {
  const form = useForm({ initialValues: initialProfile });
  const tags = form.fieldArray("tags");
  const keysOf = () => tags.entries.value.map((entry) => entry.key);
  const initialKeys = keysOf();
  assert.deepEqual(
    tags.entries.value.map((entry) => entry.path),
    ["tags.0", "tags.1", "tags.2"],
  );

  form.setErrors({ "tags.0": ["first"], "tags.2": ["third"] });
  tags.move(0, 2);
  assert.deepEqual(form.values.value.tags, ["b", "c", "a"]);
  assert.deepEqual(form.errors.value, { "tags.2": ["first"], "tags.1": ["third"] });
  assert.deepEqual(keysOf(), [initialKeys[1], initialKeys[2], initialKeys[0]]);

  tags.remove(1);
  assert.deepEqual(form.values.value.tags, ["b", "a"]);
  assert.deepEqual(form.errors.value, { "tags.1": ["first"] });

  tags.append("d");
  tags.prepend("z");
  tags.insert(1, "y");
  assert.deepEqual(form.values.value.tags, ["z", "y", "b", "a", "d"]);
  assert.deepEqual(form.errors.value, { "tags.3": ["first"] });

  tags.swap(0, 4);
  assert.deepEqual(form.values.value.tags, ["d", "y", "b", "a", "z"]);
  tags.replace(["only"]);
  assert.deepEqual(form.values.value.tags, ["only"]);
  assert.deepEqual(form.errors.value, {});
  assert.equal(new Set(keysOf()).size, 1);
});

void test("resets, replaces, and merges values", async () => {
  const form = useForm({ initialValues: initialProfile() });
  form.setValue("name", "changed", { touch: true, validate: false });
  form.setErrors({ name: ["x"], "": ["form"] });

  form.field("name").reset();
  assert.equal(form.values.value.name, "");
  assert.equal(form.isTouched("name"), false);
  assert.deepEqual(form.errors.value, { "": ["form"] });

  form.setValues({ age: 42 }, { merge: true });
  assert.equal(form.values.value.age, 42);
  form.setValues({ name: "replaced" });
  assert.equal(form.values.value.age, 20, "non-merge starts from the initial values");
  assert.equal(form.getValue("name"), "replaced");

  form.clearErrors("");
  assert.equal(form.valid.value, true);

  form.reset({ ...initialProfile(), name: "new initial" });
  assert.equal(form.values.value.name, "new initial");
  assert.equal(form.dirty.value, false);
  assert.equal(form.submitCount.value, 0);
});

void test("aborts pending validation when the scope stops", async () => {
  let signal: AbortSignal | undefined;
  const scope = effectScope();
  const form = scope.run(() =>
    useForm({
      initialValues: initialProfile,
      validators: {
        name: (_name, context) => {
          signal = context.signal;
          return new Promise<undefined>(() => undefined);
        },
      },
    }),
  );
  assert.ok(form);
  void form.validate();
  scope.stop();
  assert.equal(signal?.aborted, true);
});

void test("initial values are cloned, not shared", () => {
  const shared = initialProfile();
  const form = useForm({ initialValues: shared });
  form.setValue("address.city", "Kyoto");
  assert.equal(shared.address.city, "Tokyo");
});

void test("server rendering renders initial state without validating", async () => {
  let validated = false;
  const state = await renderComposableOnServer(() => {
    const form = useForm({
      initialValues: initialProfile,
      validators: {
        name: () => {
          validated = true;
          return "Required";
        },
      },
    });
    return {
      city: form.field("address.city").value,
      tags: form.fieldArray("tags").entries.value.map((entry) => entry.key),
      valid: form.valid,
      dirty: form.dirty,
    };
  });
  assert.equal(state, '{"city":"Tokyo","tags":[0,1,2],"valid":true,"dirty":false}');
  assert.equal(validated, false);
});

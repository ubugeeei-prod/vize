import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import type { StandardSchemaResult, StandardSchemaV1 } from "./standard-schema.ts";
import { useForm } from "./use-form.ts";

interface Profile {
  name: string;
  age: number;
  address: { city: string; zip: string };
  tags: string[];
}

const initialProfile = (): Profile => ({
  name: "Ada",
  age: 36,
  address: { city: "Tokyo", zip: "100" },
  tags: ["a", "b"],
});

function schema(
  validate: (value: unknown) => StandardSchemaResult<Profile>,
): StandardSchemaV1<Profile, Profile> {
  return { "~standard": { version: 1, vendor: "test", validate } };
}

async function flush(): Promise<void> {
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
}

void test("criteriaMode first keeps the earliest message in source order", async () => {
  const options = {
    initialValues: initialProfile(),
    schema: schema(() => ({ issues: [{ message: "From schema", path: [{ key: "name" }] }] })),
    validate: () => ({ name: ["From form"] }),
    validators: { name: () => "From field" },
  } as const;
  const all = useForm(options);
  assert.deepEqual((await all.validate()).status, "invalid");
  assert.deepEqual(all.errors.value.name, ["From schema", "From form", "From field"]);

  const first = useForm({ ...options, criteriaMode: "first" });
  await first.validate();
  assert.deepEqual(first.errors.value.name, ["From schema"]);

  const field = useForm({ ...options, criteriaMode: "first" });
  await field.field("name").validate();
  assert.deepEqual(field.errors.value.name, ["From schema"]);
});

void test("setError merges one path and watch tracks later edits", async () => {
  const form = useForm({ initialValues: initialProfile() });
  form.setErrors({ name: ["kept"], age: ["old"] });
  form.setError("age", ["too young", ""]);
  form.setError("", "form");
  assert.deepEqual(form.errors.value, { name: ["kept"], age: ["too young"], "": ["form"] });
  form.setError("age", undefined);
  assert.deepEqual(form.errors.value, { name: ["kept"], "": ["form"] });

  const city = form.watch("address.city");
  assert.equal(city.value, "Tokyo");
  form.setValue("address.city", "Osaka");
  await nextTick();
  assert.equal(city.value, "Osaka");
});

void test("dirtyFields mirrors nested edits against the initial values", () => {
  const form = useForm({ initialValues: initialProfile() });
  assert.deepEqual(form.dirtyFields.value, {});
  form.setValue("name", "Grace", { validate: false });
  form.setValue("address.city", "Osaka", { validate: false });
  form.fieldArray("tags").append("c");
  assert.deepEqual(form.dirtyFields.value, {
    name: true,
    address: { city: true },
    tags: [undefined, undefined, true],
  });
  assert.equal(form.dirty.value, true);
});

void test("submit success, failure, and a thrown handler drive isSubmitSuccessful", async () => {
  const form = useForm({
    initialValues: initialProfile(),
    validators: { name: (name) => (name === "" ? "Required" : undefined) },
  });
  assert.equal(form.isSubmitSuccessful.value, false);
  assert.equal((await form.submit()).status, "valid");
  assert.equal(form.isSubmitSuccessful.value, true);

  form.setValue("name", "");
  assert.equal((await form.submit()).status, "invalid");
  assert.equal(form.isSubmitSuccessful.value, false);
  assert.equal(form.submitCount.value, 2);

  form.setValue("name", "Ada");
  await assert.rejects(form.submit(() => Promise.reject(new Error("save failed"))));
  assert.equal(form.isSubmitSuccessful.value, false);
  assert.equal(form.submitting.value, false);
});

void test("a superseded submit leaves the success flag alone", async () => {
  const resolvers: (() => void)[] = [];
  const form = useForm({
    initialValues: initialProfile(),
    validators: {
      name: () =>
        new Promise<string>((resolve) => {
          resolvers.push(() => resolve("stale"));
        }),
    },
  });
  const first = form.submit();
  const second = form.submit();
  resolvers[1]?.();
  resolvers[0]?.();
  assert.equal((await first).status, "superseded");
  assert.equal((await second).status, "invalid");
  assert.equal(form.isSubmitSuccessful.value, false);

  form.setErrors({});
  const passing = useForm({ initialValues: initialProfile() });
  assert.equal((await passing.submit()).status, "valid");
  const hung = passing.validate();
  const later = passing.validate();
  assert.equal((await hung).status, "superseded");
  assert.equal(passing.isSubmitSuccessful.value, true);
  assert.equal((await later).status, "valid");
});

void test("reset keep-flags retain dirty values, errors, touched, and submit state", async () => {
  const form = useForm({ initialValues: initialProfile });
  form.setValue("name", "Grace", { touch: true, validate: false });
  form.setValue("address.zip", "999", { validate: false });
  await form.submit();
  form.setErrors({ name: ["x"] });
  assert.equal(form.isSubmitSuccessful.value, true);

  form.reset(undefined, {
    keepDirtyValues: true,
    keepErrors: true,
    keepTouched: true,
    keepSubmitCount: true,
    keepIsSubmitSuccessful: true,
  });
  assert.equal(form.getValue("name"), "Grace");
  assert.equal(form.getValue("address.zip"), "999");
  assert.equal(form.getValue("address.city"), "Tokyo");
  assert.equal(form.getValue("age"), 36);
  assert.deepEqual(form.errors.value, { name: ["x"] });
  assert.equal(form.isTouched("name"), true);
  assert.equal(form.submitCount.value, 1);
  assert.equal(form.isSubmitSuccessful.value, true);
  assert.equal(form.dirty.value, true);

  form.reset({ ...initialProfile(), age: 1 }, { keepDirtyValues: true });
  assert.equal(form.getValue("name"), "Grace");
  assert.equal(form.getValue("age"), 1);
  assert.equal(form.dirty.value, true);
  assert.deepEqual(form.errors.value, {});
  assert.equal(form.submitCount.value, 0);
  assert.equal(form.isSubmitSuccessful.value, false);
});

void test("disabled blocks writes and submission and still allows setError and reset", async () => {
  const scope = effectScope();
  const disabled = ref(false);
  const form = scope.run(() => useForm({ initialValues: initialProfile(), disabled }));
  assert.ok(form);
  let submitted = 0;
  disabled.value = true;
  await nextTick();
  assert.equal(form.disabled.value, true);
  form.setValue("name", "Grace");
  form.field("name").value.value = "Grace";
  form.field("name").reset();
  form.fieldArray("tags").append("c");
  form.fieldArray("tags").remove(0);
  form.setValues({ age: 1 });
  assert.equal(form.getValue("name"), "Ada");
  assert.deepEqual(form.getValue("tags"), ["a", "b"]);
  assert.equal(form.getValue("age"), 36);
  const blocked = await form.submit(() => {
    submitted += 1;
  });
  assert.deepEqual(blocked, { status: "invalid", errors: {} });
  assert.equal(submitted, 0);
  assert.equal(form.submitCount.value, 0);

  form.setError("name", "server");
  assert.deepEqual(form.errors.value, { name: ["server"] });
  form.reset();
  assert.equal(form.getValue("name"), "Ada");
  assert.deepEqual(form.errors.value, {});
  scope.stop();
});

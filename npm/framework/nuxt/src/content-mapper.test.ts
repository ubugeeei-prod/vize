import assert from "node:assert/strict";
import test from "node:test";

import { injectVizeContentMapper, registerNuxtContentMapper } from "./content-mapper.ts";

const expectedMapper = { package: "vize", extensions: [".vue"] };

void test("injects the Vize mapper once without changing existing entries", () => {
  const existing = { package: "vize", extensions: [".md"], options: { strict: true } };
  const tsConfig: Record<string, unknown> = {
    compilerOptions: { strict: true },
    contentMappers: [existing],
  };

  assert.equal(injectVizeContentMapper(tsConfig), true);
  assert.deepEqual(tsConfig.contentMappers, [existing, expectedMapper]);
  assert.equal((tsConfig.contentMappers as unknown[])[0], existing);

  assert.equal(injectVizeContentMapper(tsConfig), false);
  assert.deepEqual(tsConfig.contentMappers, [existing, expectedMapper]);
});

void test("keeps malformed array entries while adding the valid default", () => {
  const malformed = { package: "vize", extensions: "not-an-array", options: { strict: false } };
  const tsConfig = { contentMappers: [malformed] };

  assert.equal(injectVizeContentMapper(tsConfig), true);
  assert.deepEqual(tsConfig.contentMappers, [malformed, expectedMapper]);
  assert.equal(tsConfig.contentMappers[0], malformed);
});

void test("preserves an explicitly configured Vue mapper", () => {
  const configured = [
    { package: "custom-vue-mapper", extensions: [".vue"], options: { mode: "strict" } },
    { package: "vize", extensions: [".md"] },
  ];
  const tsConfig = { contentMappers: configured };

  assert.equal(injectVizeContentMapper(tsConfig), false);
  assert.equal(tsConfig.contentMappers, configured);
  assert.deepEqual(tsConfig.contentMappers, configured);
});

void test("does not overwrite a malformed contentMappers value", () => {
  for (const contentMappers of [null, "inherited", { package: "vize" }]) {
    const tsConfig = { contentMappers };
    assert.equal(injectVizeContentMapper(tsConfig), false);
    assert.equal(tsConfig.contentMappers, contentMappers);
  }
});

void test("registers prepare:types for Nuxt 3 and 4 only", () => {
  for (const nuxtMajor of [3, 4]) {
    let hookName: string | undefined;
    let hook: ((payload: unknown) => unknown) | undefined;
    registerNuxtContentMapper(
      {
        hook(name, callback) {
          hookName = name;
          hook = callback;
        },
      },
      nuxtMajor,
    );

    assert.equal(hookName, "prepare:types");
    const tsConfig = { compilerOptions: { strict: true } };
    hook?.({ tsConfig });
    hook?.({ tsConfig });
    assert.deepEqual(tsConfig, {
      compilerOptions: { strict: true },
      contentMappers: [expectedMapper],
    });
  }

  let registered = false;
  registerNuxtContentMapper(
    {
      hook() {
        registered = true;
      },
    },
    2,
  );
  assert.equal(registered, false);
});

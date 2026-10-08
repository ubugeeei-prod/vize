import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveNuxtCompilerOptions, splitNuxtCompilerDefaults } from "./compiler-option-bridge.ts";
import { mergeCompilerOptions } from "../../../builder/vite/src/plugin/compiler-config.ts";
import { createFileCompilerOptions } from "../../../builder/vite/src/plugin/compiler-scopes.ts";
import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";

await test("Nuxt defaults yield to top-level, scoped, explicit and template compiler values", () => {
  const resolved = resolveNuxtCompilerOptions(
    "/repo",
    "/",
    "/_nuxt/",
    true,
    {},
    { whitespace: "preserve" },
  );
  assert.notEqual(resolved, false);
  const original = structuredClone(resolved);
  const split = splitNuxtCompilerDefaults(resolved as VizeNuxtCompilerOptions, true, {
    whitespace: "preserve",
  });
  assert.equal(split.options.whitespace, undefined);
  assert.deepEqual(split.defaults, { whitespace: "preserve" });
  const config = {
    compiler: { whitespace: "preserve" as const },
    entries: [{ files: ["app/tight/**"], compiler: { whitespace: "condense" as const } }],
  };
  const file = createFileCompilerOptions(config, "/repo", split.options, split.defaults)!;
  assert.equal(file("/repo/app/tight/Row.vue").whitespace, "condense");
  assert.equal(file("/repo/app/loose/Row.vue").whitespace, "preserve");
  assert.equal(mergeCompilerOptions(split.options, null, split.defaults).whitespace, "preserve");
  assert.equal(
    mergeCompilerOptions(
      split.options,
      { compiler: { whitespace: "condense" }, entries: [] },
      split.defaults,
    ).whitespace,
    "condense",
  );
  const explicit = splitNuxtCompilerDefaults(
    resolved as VizeNuxtCompilerOptions,
    { whitespace: "preserve" },
    { whitespace: "condense" },
  );
  assert.equal(
    createFileCompilerOptions(config, "/repo", explicit.options, explicit.defaults)!(
      "/repo/app/tight/Row.vue",
    ).whitespace,
    "preserve",
  );
  const nested = { template: { compilerOptions: { whitespace: "preserve" as const } } };
  assert.deepEqual(mergeCompilerOptions(nested, config, split.defaults).template?.compilerOptions, {
    cacheHandlers: undefined,
    hoistStatic: undefined,
    prefixIdentifiers: undefined,
    whitespace: "preserve",
  });
  assert.deepEqual(
    resolved,
    original,
    "forwarding must not mutate host fallback or inspector options",
  );
  assert.deepEqual(
    splitNuxtCompilerDefaults(original as VizeNuxtCompilerOptions, true, undefined),
    { options: original, defaults: {} },
  );
  assert.equal(mergeCompilerOptions({}, null).whitespace, undefined);
});

await test("explicit undefined still admits the Nuxt default; unsupported/legacy host resolution stays intact", () => {
  const compiler = { whitespace: undefined };
  const resolved = resolveNuxtCompilerOptions(
    "/repo",
    "/",
    "/_nuxt/",
    compiler,
    {},
    { whitespace: "preserve" },
  );
  const split = splitNuxtCompilerDefaults(resolved as VizeNuxtCompilerOptions, compiler, {
    whitespace: "preserve",
  });
  assert.equal(mergeCompilerOptions(split.options, null, split.defaults).whitespace, "preserve");
  const host = resolveNuxtCompilerOptions(
    "/repo",
    "/",
    "/_nuxt/",
    true,
    {},
    { whitespace: "preserve", comments: true },
  );
  assert.equal((host as VizeNuxtCompilerOptions).compatibility?.hostCompiler, true);
  assert.equal((host as VizeNuxtCompilerOptions).whitespace, "preserve");
  const legacy = resolveNuxtCompilerOptions(
    "/repo",
    "/",
    "/_nuxt/",
    true,
    { vueVersion: 2 },
    { whitespace: "preserve" },
  );
  assert.equal((legacy as VizeNuxtCompilerOptions).compatibility?.hostCompiler, true);
  assert.equal((legacy as VizeNuxtCompilerOptions).whitespace, "preserve");
});

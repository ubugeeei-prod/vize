import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
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

await test("inline native compiler policies remain typed and retain scope precedence", () => {
  const config = {
    compiler: { whitespace: "preserve" },
    entries: [{ files: ["app/tight/**"], compiler: { whitespace: "condense" } }],
  } satisfies NonNullable<VizeNuxtCompilerOptions["config"]>;
  const authored: VizeNuxtCompilerOptions = { config, whitespace: "preserve" };
  const resolved = resolveNuxtCompilerOptions(
    "/repo",
    "/",
    "/_nuxt/",
    authored,
    {},
    { whitespace: "condense" },
  );
  assert.notEqual(resolved, false);
  const split = splitNuxtCompilerDefaults(resolved as VizeNuxtCompilerOptions, authored, {
    whitespace: "condense",
  });
  assert.equal(split.options.config, config);
  assert.equal(split.options.root, "/repo");
  assert.equal(
    createFileCompilerOptions(config, "/repo", split.options, split.defaults)!(
      "/repo/app/tight/Row.vue",
    ).whitespace,
    "preserve",
  );
});

await test("consumer compiler config uses the public native option types", () => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-config-types-"));
  try {
    const filename = path.join(directory, "consumer.mts");
    const options = fileURLToPath(new URL("./compiler-options.ts", import.meta.url));
    writeFileSync(
      filename,
      `
import type { VizeNuxtCompilerOptions } from ${JSON.stringify(options)};
const inline: VizeNuxtCompilerOptions = { config: { compiler: { whitespace: "preserve" }, entries: [{ files: ["app/tight/**"], compiler: { whitespace: "condense" } }] } };
const list: VizeNuxtCompilerOptions = { config: [{ compiler: { whitespace: "preserve" } }] };
const factory: VizeNuxtCompilerOptions = { config: async (env) => ({ compiler: { whitespace: env.command === "serve" ? "preserve" : "condense" } }) };
// @ts-expect-error Native whitespace leaves retain their registered values.
const invalid: VizeNuxtCompilerOptions = { config: { compiler: { whitespace: "invalid" } } };
// @ts-expect-error Native policy does not introduce custom config inheritance.
const inherited: VizeNuxtCompilerOptions = { config: { extends: "./base" } };
`,
    );
    const program = ts.createProgram([filename], {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
      allowImportingTsExtensions: true,
    });
    assert.deepEqual(
      ts
        .getPreEmitDiagnostics(program)
        .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
      [],
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

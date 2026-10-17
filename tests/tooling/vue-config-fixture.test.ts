import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import {
  inlineVuePluginFixtureOptions,
  nativeConfigToVueOptions,
  vueConfigFixtureSource,
  writeVueConfigFixture,
} from "./support/vue-config-fixture.ts";

test("explicit inline fixtures retain scoped compiler settings and module options without canonical rewrites", () => {
  const native = {
    vue: { version: "2.7" },
    lsp: { formatting: false },
    compiler: { whitespace: "preserve" },
    entries: [{ files: ["app/components/tight/**"], compiler: { whitespace: "condense" } }],
  };
  const moduleOptions = { whitespace: "condense", experimental: { hmr: false } };
  const bytes = JSON.stringify(native);
  for (const document of [native, bytes, new TextEncoder().encode(bytes)]) {
    const options = inlineVuePluginFixtureOptions(moduleOptions, document);
    assert.ok(options !== false);
    assert.deepEqual(options, {
      whitespace: "condense",
      experimental: { hmr: false },
      config: {
        vue: { version: "2.7" },
        lsp: { formatting: false },
        compiler: { whitespace: "preserve" },
        entries: [{ files: ["app/components/tight/**"], compiler: { whitespace: "condense" } }],
      },
    });
    assert.notEqual(options, moduleOptions);
    assert.notEqual(options.config, native);
    const config = options.config as typeof native;
    config.entries[0].compiler.whitespace = "changed";
    (options.experimental as typeof moduleOptions.experimental).hmr = true;
    assert.equal(native.entries[0].compiler.whitespace, "condense");
    assert.equal(moduleOptions.experimental.hmr, false);
  }
  assert.equal(JSON.stringify(native), bytes);
});

test("disabled inline modules ignore config documents and absent documents preserve module options", () => {
  assert.equal(inlineVuePluginFixtureOptions(false, "not JSON"), false);
  for (const document of [null, undefined]) {
    assert.deepEqual(inlineVuePluginFixtureOptions(undefined, document), {});
    assert.deepEqual(inlineVuePluginFixtureOptions({ whitespace: "preserve" }, document), {
      whitespace: "preserve",
    });
  }
});

test("native fixture carrier preserves independently specified native settings through the real projector", async () => {
  const native = {
    name: "native-custody",
    vue: { version: "2.7" },
    compiler: { ssr: true, whitespace: "preserve", sourceMap: false },
    typeChecker: { enabled: false, strict: false },
    lsp: { formatting: false, semanticTokens: true, lint: false },
    formatter: { semi: false, singleQuote: true, tabWidth: 4, sortImports: false },
    files: ["src/**/*.vue"],
    ignores: ["generated/**"],
    linter: { preset: "essential", rules: { "a11y/alt-text": "off" } },
    entries: [
      {
        basePath: "packages/ui",
        files: ["**/*.vue"],
        ignores: ["vendor/**"],
        linter: { rules: { "a11y/alt-text": "error" } },
      },
    ],
  };
  const original = JSON.stringify(native);
  const expected = {
    name: "native-custody",
    vue: { version: "2.7" },
    compiler: { ssr: true, whitespace: "preserve", sourceMap: false },
    typeChecker: { jsxTypecheck: false, enabled: false, strict: false },
    languageServer: { formatting: false, semanticTokens: true, lint: false },
    formatter: { semi: false, singleQuote: true, tabWidth: 4, sortImports: false },
    files: ["src/**/*.vue"],
    ignores: ["generated/**"],
    linter: { preset: "essential", rules: { "a11y/alt-text": "off" } },
    entries: [
      {
        basePath: "packages/ui",
        files: ["**/*.vue"],
        ignores: ["vendor/**"],
        linter: { rules: { "a11y/alt-text": "error" } },
      },
    ],
  };
  for (const document of [native, original, new TextEncoder().encode(original)]) {
    const source = vueConfigFixtureSource(document);
    const { default: exported } = await import(
      `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`
    );
    assert.deepEqual(await projectViteConfigSource(exported), expected);
  }
  assert.equal(JSON.stringify(native), original);
});

test("native defaults and explicit JSX choices retain their original meaning", async () => {
  for (const [native, expected] of [
    [{}, { typeChecker: { jsxTypecheck: false } }],
    [{ typeChecker: { jsxTypecheck: true } }, { typeChecker: { jsxTypecheck: true } }],
    [{ typeChecker: { jsxTypecheck: false } }, { typeChecker: { jsxTypecheck: false } }],
  ]) {
    assert.deepEqual(
      await projectViteConfigSource({ vue: nativeConfigToVueOptions(native) }),
      expected,
    );
  }
});

test("document schema references remain inert carrier metadata rather than Vue options", async () => {
  for (const $schema of [
    "node_modules/.vize/vize.config.schema.json",
    "https://example.test/schema.json",
    true,
  ]) {
    const native = { $schema, formatter: { semi: false } };
    const { default: exported } = await import(
      `data:text/javascript;base64,${Buffer.from(vueConfigFixtureSource(native)).toString("base64")}`
    );
    assert.equal(exported.$schema, $schema);
    assert.equal(Object.hasOwn(exported.vue, "$schema"), false);
    assert.deepEqual(await projectViteConfigSource(exported), {
      typeChecker: { jsxTypecheck: false },
      formatter: { semi: false },
    });
    assert.equal(native.$schema, $schema);
  }
});

test("retired inheritance, duplicate ownership and malformed native roots fail visibly", () => {
  for (const native of [
    { extends: "./base.json" },
    { entries: [{ files: ["*.vue"], extends: "./base.json" }] },
  ]) {
    assert.throws(() => nativeConfigToVueOptions(native), /Custom config extends/);
  }
  assert.throws(
    () => nativeConfigToVueOptions({ vue: { version: "3" }, version: "2" }),
    /Duplicate Vue setting/,
  );
  assert.throws(
    () => nativeConfigToVueOptions({ lsp: {}, languageServer: {} }),
    /Duplicate languageServer/,
  );
  for (const json of ["null", "[]", '"invalid"']) {
    assert.throws(() => nativeConfigToVueOptions(json), /native configuration object/);
  }
});

test("writer emits a genuine discoverable module while source fixture bytes stay intact", async () => {
  const root = mkdtempSync(join(tmpdir(), "vize-canonical-config-fixture-"));
  try {
    const native = '{"vue":{"version":"3"},"lsp":{"hover":false}}';
    const path = writeVueConfigFixture(root, native);
    assert.equal(path, join(root, "vite.config.mjs"));
    assert.equal(readFileSync(path, "utf8"), vueConfigFixtureSource(native));
    const { default: exported } = await import(pathToFileURL(path).href);
    assert.deepEqual(await projectViteConfigSource(exported), {
      vue: { version: "3" },
      languageServer: { hover: false },
      typeChecker: { jsxTypecheck: false },
    });
    assert.equal(native, '{"vue":{"version":"3"},"lsp":{"hover":false}}');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("authored parity projects retain their original native configuration policy through Vite", async () => {
  for (const [name, original] of [
    ["options-api", {}],
    ["class-component", {}],
    ["vue2-filter-build", { compiler: { compatibility: { vueVersion: "2" } } }],
  ] as const) {
    const root = new URL(`../_fixtures/_projects/${name}/`, import.meta.url);
    const archived = readFileSync(new URL("vize.config.json", root));
    assert.deepEqual(JSON.parse(archived.toString()), original);
    const { default: exported } = await import(new URL("vite.config.mjs", root).href);
    assert.deepEqual(await projectViteConfigSource(exported), {
      ...original,
      typeChecker: { jsxTypecheck: false },
    });
    assert.deepEqual(readFileSync(new URL("vize.config.json", root)), archived);
  }
});

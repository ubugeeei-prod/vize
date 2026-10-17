import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import {
  projectViteConfigSource,
  resolveViteConfigExport,
  vueConfigOptions,
} from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

test("the complete canonical validation graph derives from the authoritative native schema", async () => {
  const authoritative = JSON.parse(
    await readFile(
      new URL("../../npm/cli/schemas/vize.config.schema.json", import.meta.url),
      "utf8",
    ),
  );
  const body = await readFile(
    new URL("../../crates/vize_carton/src/config/loader/vite-runtime.mjs", import.meta.url),
    "utf8",
  );
  const encoded =
    /const vueOptionSchema = JSON.parse\(\s*((?:"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'))\s*,?\s*\);/u.exec(
      body,
    );
  assert.ok(encoded);
  // The matched expression is one complete string literal, regardless of formatter quoting.
  const actual = JSON.parse(runInNewContext(encoded[1], Object.create(null), { timeout: 100 }));
  const runtime = await readFile(
    new URL("../../npm/cli/src/types/runtime.ts", import.meta.url),
    "utf8",
  );
  const optionKeys = /type VueOptionKeys =([\s\S]*?);/u.exec(runtime);
  assert.ok(optionKeys);
  const keys = [...optionKeys[1].matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
  const entryType = /export type VueOptionsEntry = Pick<\s*VizeConfigEntry,\s*([^>]+)>;/u.exec(
    runtime,
  );
  assert.ok(entryType);
  const scopedKeys = [...entryType[1].matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
  assert.deepEqual(scopedKeys.toSorted(), ["basePath", "files", "ignores", "linter"].toSorted());
  const fields = Object.fromEntries(keys.map((key) => [key, authoritative.properties[key]]));
  fields.version = { $ref: "#/definitions/VueVersion" };
  fields.entries = {
    type: "array",
    items: {
      type: "object",
      properties: Object.fromEntries(
        scopedKeys.map((key) => [key, authoritative.definitions.VizeConfigEntry.properties[key]]),
      ),
      additionalProperties: false,
    },
  };
  const definitions = {};
  const scan = (value) => {
    if (value && typeof value === "object") {
      if (value.$ref) {
        const key = value.$ref.split("/").at(-1);
        if (!(key in definitions)) {
          definitions[key] = authoritative.definitions[key];
          scan(definitions[key]);
        }
      }
      Object.values(value).forEach(scan);
    }
  };
  scan(fields);
  const omitAnnotations = (value) =>
    Array.isArray(value)
      ? value.map(omitAnnotations)
      : value && typeof value === "object"
        ? Object.fromEntries(
            Object.entries(value)
              .filter(([key]) => !["description", "default", "title", "$schema"].includes(key))
              .map(([key, child]) => [key, omitAnnotations(child)]),
          )
        : value;
  assert.deepEqual(
    actual,
    omitAnnotations({
      type: "object",
      properties: fields,
      additionalProperties: false,
      definitions,
    }),
  );
});

test("canonical namespaces refuse unknown roots, entries, native fields and mismatched section shapes", () => {
  for (const vue of [
    null,
    false,
    [],
    () => ({}),
    new Date(),
    Promise.resolve({}),
    { typecheck: {} },
    { unknownNative: undefined },
    { compiler: false },
    { typeChecker: [] },
    { linter: "essential" },
    { formatter: true },
    { languageServer: null },
    { globalTypes: 1 },
    { files: "**/*.vue" },
    { ignores: [null] },
    { basePath: 4 },
    { version: 3 },
    { version: "legacy" },
    { compiler: { enabled: false } },
    { typeChecker: { strcit: true } },
    { linter: { rules: { "vue/no-v-html": "warning" } } },
    { formatter: { sortImports: true } },
    { formatter: { printWidth: 1.5 } },
    { languageServer: { enabled: "false" } },
    { entries: {} },
    { entries: [null] },
    { entries: [{ lib: { dir: "scoped" } }] },
    { entries: [{ unknown: true }] },
    { entries: [{ version: "4" }] },
    { entries: [{ extends: "./old.json" }] },
    { entries: [{ files: [undefined] }] },
    { lib: { registries: { invalidNamespace: "npm:somewhere" } } },
    { lib: { registries: { "@team": { dir: "components" } } } },
    { typeChecker: { lspRequestTimeoutMs: 0 } },
    { linter: { maxWarnings: -1 } },
  ])
    assert.throws(() => vueConfigOptions({ vue }), TypeError);
  const sparse = [];
  sparse.length = 1;
  assert.throws(() => vueConfigOptions({ vue: { entries: sparse } }), TypeError);
});

test("Corsa server counts use the CLI range in canonical config validation", () => {
  for (const servers of [1, 2, 32]) {
    assert.deepEqual(vueConfigOptions({ vue: { typeChecker: { servers } } }), {
      typeChecker: { servers },
    });
  }
  for (const servers of [0, -1, 33, 1.5]) {
    assert.throws(() => vueConfigOptions({ vue: { typeChecker: { servers } } }), TypeError);
  }
});

test("all documented canonical native sections accept their original field values and own scopes", async () => {
  const vue = {
    version: "3",
    compiler: { vapor: false, customElements: ["my-*"], whitespace: "preserve" },
    typeChecker: { enabled: false, lspRequestTimeoutMs: 1, servers: 1 },
    linter: {
      enabled: false,
      maxWarnings: 0,
      categories: { correctness: "error" },
      rules: { "vue/no-v-html": "off" },
    },
    formatter: { sortImports: false, attributeGroups: [["id"], ["class", "style"]] },
    languageServer: { enabled: false, formatting: false, typecheck: false },
    globalTypes: { GLOBAL: "string" },
    files: [],
    ignores: [],
    basePath: ".",
    lib: { registries: { "@team": { source: "npm:widgets", dir: "components" } } },
    entries: [
      {
        files: ["src/**"],
        ignores: ["generated/**"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  };
  assert.equal(vueConfigOptions({ vue }), vue);
  const { version: _version, ...expected } = vue;
  assert.deepEqual(await projectViteConfigSource({ vue }), {
    ...expected,
    vue: { version: "3" },
    typeChecker: { jsxTypecheck: true, enabled: false, lspRequestTimeoutMs: 1, servers: 1 },
    entries: [
      {
        files: ["src/**"],
        ignores: ["generated/**"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  });
});

test("retired namespaces fail without invoking any legacy or Vite hooks", async () => {
  let calls = 0;
  const vize = () => {
    calls++;
    return {};
  };
  assert.equal(vueConfigOptions({ vize }), undefined);
  await assert.rejects(
    projectViteConfigSource({ vue: { compiler: false }, vize }),
    /vize: configuration is no longer supported/,
  );
  assert.equal(calls, 0);
  await assert.rejects(
    projectViteConfigSource({ vize: { packageLocalUnknown: { active: true } } }),
    /move Vue settings to vue:/,
  );
  assert.equal(calls, 0);
});

test("canonical entries admit only real native linter scopes while all other native sections stay at the root", async () => {
  const rootOptions = {
    version: "3",
    compiler: { vapor: false },
    typeChecker: { strict: false },
    formatter: { useTabs: false },
    languageServer: { enabled: false },
    globalTypes: { Shared: "boolean" },
    lib: { dir: "src/lib" },
  };
  const scoped = {
    basePath: "packages/app",
    files: ["*.vue"],
    ignores: ["Ignored.vue"],
    linter: { rules: { "vue/no-v-html": "error" } },
  };
  const source = { vue: { ...rootOptions, entries: [scoped] } };
  assert.equal(vueConfigOptions(source), source.vue);
  const { version: _version, ...flat } = rootOptions;
  assert.deepEqual(await projectViteConfigSource(source), {
    ...flat,
    typeChecker: { jsxTypecheck: true, strict: false },
    vue: { version: "3" },
    entries: [scoped],
  });
  for (const [key, value] of Object.entries(rootOptions)) {
    for (const item of [value, undefined]) {
      const vue = { entries: [{ ...scoped, [key]: item }] };
      const error = new RegExp("vue\\.entries\\[0\\]: unknown option " + key);
      assert.throws(() => vueConfigOptions({ vue }), error);
      await assert.rejects(projectViteConfigSource({ vue }), error);
    }
  }
  let callbacks = 0;
  await assert.rejects(
    projectViteConfigSource({
      vue: { entries: [{ files: ["src/**"], version: "2" }] },
      vize() {
        callbacks++;
        return {};
      },
    }),
    /vize: configuration is no longer supported/u,
  );
  assert.equal(callbacks, 0);

  // Preserve the old fixture as history; retired ingress now requires migration.
  const original = await readFile(
    new URL(
      "../_fixtures/differential/config/vue-options/original-vite-runtime.mjs.fixture",
      import.meta.url,
    ),
  );
  const legacy = await import("data:text/javascript;base64," + original.toString("base64"));
  const old = { vize: { entries: [{ ...scoped, ...rootOptions, unknownScopedOwner: true }] } };
  assert.ok(await legacy.resolveViteConfigExport(old));
  await assert.rejects(projectViteConfigSource(old), /move Vue settings to vue:/);
});

test("new public numeric Vue 0.10 ingress projects native enums and respects canonical version leaves", async () => {
  const schema = JSON.parse(
    await readFile(
      new URL("../../npm/cli/schemas/vize.config.schema.json", import.meta.url),
      "utf8",
    ),
  );
  const pairs = [
    [3, "3"],
    ["2.7", "2.7"],
    [2, "2"],
    [1, "1"],
    [0.11, "0.11"],
    [0.1, "0.10"],
  ];
  assert.deepEqual(
    pairs.map(([, native]) => native),
    schema.definitions.VueVersion.enum,
  );
  for (const [vueVersion, version] of pairs) {
    const source = { compiler: { vueVersion } };
    const expected = {
      typeChecker: { jsxTypecheck: true },
      compiler: { compatibility: { vueVersion: version } },
    };
    assert.deepEqual(await projectViteConfigSource(source), expected);
    assert.deepEqual(await projectViteConfigSource({ ...source, vue: { version } }), {
      ...expected,
      vue: { version },
    });
  }
  const compiler = { vueVersion: 0.1 };
  assert.deepEqual(await projectViteConfigSource({ compiler, vue: { version: "3" } }), {
    typeChecker: { jsxTypecheck: true },
    compiler: { compatibility: { vueVersion: "0.10" } },
    vue: { version: "3" },
  });
  assert.deepEqual(
    await projectViteConfigSource({
      compiler,
      vue: { version: "2", compiler: { compatibility: { vueVersion: "1" } } },
    }),
    {
      typeChecker: { jsxTypecheck: true },
      compiler: { compatibility: { vueVersion: "1" } },
      vue: { version: "2" },
    },
  );
  assert.equal(
    (await projectViteConfigSource({ compiler: { vueVersion: "legacy" } })).compiler.compatibility
      .vueVersion,
    "2",
  );
});

test("evaluated fluent output retains native policy without evaluating Vite plugins or tasks again", async () => {
  const source = {
    vue: { musea: { include: ["stories/*.art.vue"] }, formatter: { singleQuote: true } },
  };
  const processed = {
    plugins: [
      () => {
        throw new Error("plugin evaluated");
      },
    ],
    [Symbol.for("@vizejs/vite-plugin/vite-plus/source")]: source,
  };
  assert.deepEqual(await resolveViteConfigExport(processed), await resolveViteConfigExport(source));
});

test("every native language-server boolean survives canonical projection, including crossFile opt-outs", async () => {
  const model = await readFile(
    new URL("../../davinci/vize_l0/src/config/model/language_server.rs", import.meta.url),
    "utf8",
  );
  const fields = [...model.matchAll(/pub ([a-z_]+): Option<bool>/gu)].map((match) =>
    match[1].replace(/_([a-z])/gu, (_, letter) => letter.toUpperCase()),
  );
  assert.ok(fields.includes("crossFile"));
  assert.ok(fields.includes("ecosystem"));
  assert.ok(fields.includes("signatureHelp"));
  for (const enabled of [true, false]) {
    const languageServer = Object.fromEntries(fields.map((field) => [field, enabled]));
    assert.deepEqual(await projectViteConfigSource({ vue: { languageServer } }), {
      typeChecker: { jsxTypecheck: true },
      languageServer,
    });
  }
  await assert.rejects(
    projectViteConfigSource({ vue: { languageServer: { crossFileAnalysis: true } } }),
    /vue\.languageServer: unknown option crossFileAnalysis/u,
  );
});

test("canonical lint preset names match the native parser, including accepted aliases", async () => {
  const native = await readFile(
    new URL("../../crates/vize_patina/src/preset.rs", import.meta.url),
    "utf8",
  );
  const parse = /pub fn parse\(value: &str\) -> Option<Self> \{([\s\S]*?)\n    \}/u.exec(native);
  assert.ok(parse);
  const accepted = [...parse[1].matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
  assert.ok(accepted.includes("happy-path"));
  assert.ok(accepted.includes("recommended"));
  assert.ok(accepted.includes("all"));
  const schema = JSON.parse(
    await readFile(
      new URL("../../npm/cli/schemas/vize.config.schema.json", import.meta.url),
      "utf8",
    ),
  );
  assert.deepEqual(
    schema.definitions.LinterConfig.properties.preset.enum.toSorted(),
    accepted.toSorted(),
  );
  const generated = await readFile(
    new URL("../../npm/cli/src/types/generated.ts", import.meta.url),
    "utf8",
  );
  const presetType = /export interface LinterConfig \{[\s\S]*?\bpreset\?:\s*([\s\S]*?);/u.exec(
    generated,
  );
  assert.ok(presetType);
  assert.deepEqual(
    [...presetType[1].matchAll(/"([^"]+)"/gu)].map((match) => match[1]).toSorted(),
    accepted.toSorted(),
  );
  for (const preset of accepted) {
    const result = await projectViteConfigSource({ vue: { linter: { preset } } });
    assert.deepEqual(result.linter, { preset });
  }
  for (const preset of ["", "Recommended", "essentials", "not-a-preset"]) {
    await assert.rejects(
      projectViteConfigSource({ vue: { linter: { preset } } }),
      /vue\.linter\.preset: unsupported native option value/u,
    );
  }
});

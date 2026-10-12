import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { test } from "node:test";
import {
  mergeVueConfigOptions,
  projectViteConfigSource,
  resolveViteConfigExport,
  vueConfigOptions,
} from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

const fixture = new URL("../_fixtures/differential/config/vue-options/", import.meta.url);
const original = await readFile(new URL("original-vite-runtime.mjs.fixture", fixture));
assert.equal(
  createHash("sha256").update(original).digest("hex"),
  "df7a0d84772fe31572506de80fd96ee5baae8f38cf72f232a45fbaa1545b04df",
);
const legacy = await import("data:text/javascript;base64," + original.toString("base64"));
const root = resolve("/tmp/vize-vue-config-original");

test("whole original legacy settings and array vectors remain literal without canonical Vue options", async () => {
  const cases = [
    {},
    { vize: { unknownOwner: { enabled: false }, vue: { version: "2.7" } } },
    {
      vize: {
        typeChecker: { jsxTypecheck: false },
        lint: { preset: "essential" },
        languageServer: { formatting: false },
      },
      compiler: { vueVersion: 2.7, whitespace: "preserve" },
      typecheck: { strict: true },
      lint: { vize: { rules: { "a11y/img-alt": "error" }, helpLevel: "short" } },
      fmt: { printWidth: 90, vize: { singleQuote: true, tabWidth: 4 } },
    },
    {
      root: "app",
      vize: [
        { __vizeProjectRoot: "decoy", unknownOwner: 7, formatter: { singleQuote: true } },
        { files: [], typeChecker: { strict: false } },
        { basePath: "packages/ui", files: ["*.vue"] },
        { ignores: ["generated/**"] },
        { files: ["src/*.vue"], typeChecker: { tsconfig: "./tsconfig.json" } },
      ],
      typecheck: false,
    },
    { vize: { vite: { include: /\.vue$/ }, entries: [{ files: ["src/**"] }] } },
  ];
  for (const source of cases) {
    assert.deepEqual(
      await resolveViteConfigExport(source, undefined, root),
      await legacy.resolveViteConfigExport(source, undefined, root),
    );
  }
});

test("already evaluated projection invokes only one legacy callback with the exact supplied environment", async () => {
  const seen = [];
  const env = Object.freeze({
    mode: "production",
    command: "build",
    isSsrBuild: true,
    isPreview: true,
  });
  const source = {
    vue: { typeChecker: { enabled: false } },
    async vize(actual) {
      seen.push(actual);
      return { typeChecker: { strict: true }, formatter: { singleQuote: true } };
    },
  };
  assert.equal(vueConfigOptions(source), source.vue);
  assert.deepEqual(seen, []);
  assert.deepEqual(await projectViteConfigSource(source, env), {
    typeChecker: { jsxTypecheck: true, strict: true, enabled: false },
    formatter: { singleQuote: true },
  });
  assert.deepEqual(seen, [env]);
});

test("Vite export mapping preserves the original whole environment and evaluates each source once", async () => {
  for (const command of ["check", "lint", "fmt", "serve", "build"]) {
    const env = Object.freeze({ command, mode: "production", isSsrBuild: false, isPreview: true });
    const expected =
      command === "serve" || command === "build" ? env : { ...env, command: "serve" };
    const seen = [];
    const factory = async (actual) => {
      seen.push(actual);
      return {
        vue: { languageServer: { enabled: false } },
        vize: async (shared) => {
          seen.push(shared);
          return { formatter: { singleQuote: false } };
        },
      };
    };
    assert.deepEqual(await resolveViteConfigExport(factory, env), {
      typeChecker: { jsxTypecheck: true },
      formatter: { singleQuote: false },
      languageServer: { enabled: false },
    });
    assert.deepEqual(seen, [expected, expected]);
    assert.equal(seen[0], seen[1]);
    assert.equal(env.command, command);
    seen.length = 0;
    await projectViteConfigSource(
      {
        vize: (actual) => {
          seen.push(actual);
          return {};
        },
      },
      env,
    );
    assert.deepEqual(seen, [expected]);
  }
});

test("object and Promise exports agree, callbacks reject, and native projection never starts hooks", async () => {
  const fail = () => {
    throw new Error("hook must not run");
  };
  const plugin = { name: "consumer", config: fail, transform: fail };
  const source = { plugins: [plugin, Promise.resolve([plugin])], pack: { hooks: fail }, vue: {} };
  const expected = { typeChecker: { jsxTypecheck: true } };
  assert.deepEqual(await resolveViteConfigExport(source), expected);
  assert.deepEqual(await resolveViteConfigExport(Promise.resolve(source)), expected);
  const error = new Error("authored callback rejection");
  await assert.rejects(
    resolveViteConfigExport(async () => {
      throw error;
    }),
    (actual) => actual === error,
  );
  await assert.rejects(
    projectViteConfigSource({
      vize: async () => {
        throw error;
      },
    }),
    (actual) => actual === error,
  );
  assert.equal(source.plugins[0], plugin);
  assert.equal(source.pack.hooks, fail);
});

test("canonical false leaves and replacement arrays override only explicitly supplied native settings", async () => {
  const source = {
    vize: {
      typeChecker: { enabled: true, strict: true },
      languageServer: { enabled: true, lint: true },
      globalTypes: { Old: "string", Shared: "number" },
      formatter: {
        singleQuote: true,
        attributeGroups: [["old"]],
        sortImports: { groups: ["type"] },
      },
    },
    vue: {
      typeChecker: { enabled: false, strict: undefined },
      languageServer: { enabled: false },
      globalTypes: { Shared: "boolean" },
      formatter: { singleQuote: false, attributeGroups: [], sortImports: false },
    },
  };
  const before = structuredClone(source);
  assert.deepEqual(await projectViteConfigSource(source), {
    typeChecker: { jsxTypecheck: true, enabled: false, strict: true },
    languageServer: { enabled: false, lint: true },
    globalTypes: { Old: "string", Shared: "boolean" },
    formatter: { singleQuote: false, attributeGroups: [], sortImports: false },
  });
  assert.deepEqual(source, before);
});

test("canonical dialect and scoped entries project to strict native fields without losing root paths", async () => {
  assert.deepEqual(
    await projectViteConfigSource(
      {
        root: "app",
        vize: {
          entries: [{ files: ["old/**"] }],
          compiler: { compatibility: { vueVersion: "3" } },
        },
        vue: {
          version: "2.7",
          files: ["src/*.vue"],
          lib: { dir: "src/lib" },
          typeChecker: {
            tsconfig: "tsconfig.json",
            globalsFile: "types/globals.d.ts",
            corsaPath: "tools/corsa",
          },
          entries: [
            {
              basePath: "packages/ui",
              files: ["*.vue"],
              linter: { rules: { "a11y/img-alt": "warn" } },
            },
          ],
        },
      },
      undefined,
      root,
    ),
    {
      __vizeProjectRoot: resolve(root, "app"),
      basePath: resolve(root, "app"),
      files: ["src/*.vue"],
      lib: { dir: "src/lib" },
      typeChecker: {
        jsxTypecheck: true,
        tsconfig: resolve(root, "app/tsconfig.json"),
        globalsFile: resolve(root, "app/types/globals.d.ts"),
        corsaPath: resolve(root, "app/tools/corsa"),
      },
      compiler: { compatibility: { vueVersion: "3" } },
      vue: { version: "2.7" },
      entries: [
        {
          basePath: resolve(root, "app/packages/ui"),
          files: ["*.vue"],
          linter: { rules: { "a11y/img-alt": "warn" } },
        },
      ],
    },
  );
});

test("explicit canonical scoped vectors replace legacy vectors and preserve whole unscoped globals", async () => {
  const legacyEntries = [
    { name: "global", globalTypes: { Shared: "number" }, formatter: { tabWidth: 2 } },
    { files: [], formatter: { tabWidth: 4 } },
    { basePath: "old", files: ["*.vue"] },
    { ignores: ["old/**"] },
  ];
  const vue = {
    formatter: { singleQuote: true },
    entries: [{ files: ["new/**"], linter: { rules: { "vue/no-v-html": "warn" } } }],
  };
  assert.deepEqual(await projectViteConfigSource({ vize: legacyEntries, vue }), [
    { typeChecker: { jsxTypecheck: true } },
    legacyEntries[0],
    { formatter: { singleQuote: true } },
    { files: ["new/**"], linter: { rules: { "vue/no-v-html": "warn" } } },
  ]);
  assert.deepEqual(await projectViteConfigSource({ vize: legacyEntries, vue: { entries: [] } }), [
    { typeChecker: { jsxTypecheck: true } },
    legacyEntries[0],
    {},
  ]);
  assert.deepEqual(await projectViteConfigSource({ vize: legacyEntries, vue: {} }), [
    { typeChecker: { jsxTypecheck: true } },
    ...legacyEntries,
    {},
  ]);
});

test("shared canonical composition preserves explicit false and replaces all ordered array leaves", async () => {
  const base = {
    files: ["old/**"],
    entries: [{ files: ["old/**"] }],
    typeChecker: { enabled: true, strict: true },
  };
  const own = {
    files: [],
    entries: [{ files: ["new/**"] }],
    typeChecker: { enabled: false, strict: undefined },
  };
  const result = {
    files: [],
    entries: [{ files: ["new/**"] }],
    typeChecker: { enabled: false, strict: true },
  };
  assert.deepEqual(mergeVueConfigOptions(base, own), result);
  assert.equal(mergeVueConfigOptions(base, undefined), base);
  assert.equal(mergeVueConfigOptions(undefined, own), own);
  assert.equal(mergeVueConfigOptions(undefined, undefined), undefined);
  assert.deepEqual(
    await resolveViteConfigExport({
      extends: { vue: base },
      vue: own,
    }),
    {
      typeChecker: { jsxTypecheck: true, enabled: false, strict: true },
      files: [],
      entries: [{ files: ["new/**"] }],
    },
  );
});

test("source envelopes retain canonical settings through extends while cycle refusals remain", async () => {
  let calls = 0;
  const wrapped = Object.assign(
    () => {
      throw new Error("factory must not run");
    },
    {
      [Symbol.for("@vizejs/vite-plugin/vite-plus/source")]: {
        extends: async () => {
          calls++;
          return { vue: { formatter: { tabWidth: 4 }, files: ["base/**"] } };
        },
        vue: { formatter: { singleQuote: true } },
      },
    },
  );
  assert.deepEqual(await resolveViteConfigExport({ extends: wrapped, vue: undefined }), {
    typeChecker: { jsxTypecheck: true },
    files: ["base/**"],
    formatter: { tabWidth: 4, singleQuote: true },
  });
  assert.equal(calls, 1);
  const cyclic = {};
  cyclic.extends = cyclic;
  await assert.rejects(resolveViteConfigExport(cyclic), /Circular extends/);
});

test("canonical dynamic native maps preserve own prototype-shaped names without mutation", async () => {
  const vue = JSON.parse(
    '{"globalTypes":{"constructor":"boolean","__proto__":"number"},"linter":{"rules":{"__proto__":"warn"}}}',
  );
  const result = await projectViteConfigSource({ vize: { globalTypes: { Other: "string" } }, vue });
  assert.deepEqual(result, {
    typeChecker: { jsxTypecheck: true },
    globalTypes: JSON.parse('{"Other":"string","constructor":"boolean","__proto__":"number"}'),
    linter: { rules: JSON.parse('{"__proto__":"warn"}') },
  });
  assert.equal(Object.getPrototypeOf(result.globalTypes), Object.prototype);
});

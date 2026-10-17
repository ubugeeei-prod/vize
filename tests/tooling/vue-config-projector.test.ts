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

test("retired legacy settings and array vectors refuse while original source authority stays frozen", async () => {
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
    // Preserve the complete original vectors and immutable observer above.
    // Their retired carrier must refuse rather than regain a silent fallback.
    const original = await legacy.resolveViteConfigExport(source, undefined, root);
    if (Object.hasOwn(source, "vize")) {
      await assert.rejects(
        resolveViteConfigExport(source, undefined, root),
        /vize:.*no longer supported/,
      );
    } else {
      assert.deepEqual(await resolveViteConfigExport(source, undefined, root), original);
    }
  }
});

test("already evaluated projection does not repeat the source callback or change its environment", async () => {
  const seen = [];
  const env = Object.freeze({
    mode: "production",
    command: "build",
    isSsrBuild: true,
    isPreview: true,
  });
  const factory = async (actual) => {
    seen.push(actual);
    return {
      vue: {
        typeChecker: { strict: true, enabled: false },
        formatter: { singleQuote: true },
      },
    };
  };
  const source = await factory(env);
  assert.equal(vueConfigOptions(source), source.vue);
  assert.deepEqual(seen, [env]);
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
        vue: {
          languageServer: { enabled: false },
          formatter: { singleQuote: false },
        },
      };
    };
    assert.deepEqual(await resolveViteConfigExport(factory, env), {
      typeChecker: { jsxTypecheck: true },
      formatter: { singleQuote: false },
      languageServer: { enabled: false },
    });
    assert.deepEqual(seen, [expected]);
    assert.equal(env.command, command);
    seen.length = 0;
    await projectViteConfigSource(
      {
        vue: {},
      },
      env,
    );
    assert.deepEqual(seen, []);
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
  let retiredCalls = 0;
  await assert.rejects(
    projectViteConfigSource({
      vize: async () => {
        retiredCalls++;
        throw error;
      },
    }),
    /vize:.*no longer supported/,
  );
  assert.equal(retiredCalls, 0);
  assert.equal(source.plugins[0], plugin);
  assert.equal(source.pack.hooks, fail);
});

test("canonical false leaves and replacement arrays override only explicitly supplied native settings", async () => {
  const base = {
    typeChecker: { enabled: true, strict: true },
    languageServer: { enabled: true, lint: true },
    globalTypes: { Old: "string", Shared: "number" },
    formatter: {
      singleQuote: true,
      attributeGroups: [["old"]],
      sortImports: { groups: ["type"] },
    },
  };
  const own = {
    typeChecker: { enabled: false, strict: undefined },
    languageServer: { enabled: false },
    globalTypes: { Shared: "boolean" },
    formatter: { singleQuote: false, attributeGroups: [], sortImports: false },
  };
  const source = { vue: mergeVueConfigOptions(base, own) };
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
        compiler: { vueVersion: 3 },
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

test("explicit canonical scoped vectors replace inherited vectors and preserve whole unscoped globals", async () => {
  const global = { name: "global", globalTypes: { Shared: "number" }, formatter: { tabWidth: 2 } };
  const retiredScopedFormatter = { files: [], formatter: { tabWidth: 4 } };
  assert.throws(
    () => mergeVueConfigOptions({ entries: [retiredScopedFormatter] }, {}),
    /unknown option formatter/,
  );
  const entries = [{ files: [] }, { basePath: "old", files: ["*.vue"] }, { ignores: ["old/**"] }];
  const vue = {
    formatter: { singleQuote: true },
    entries: [{ files: ["new/**"], linter: { rules: { "vue/no-v-html": "warn" } } }],
  };
  const base = { ...global, entries };
  const project = (own) => projectViteConfigSource({ vue: mergeVueConfigOptions(base, own) });
  assert.deepEqual(await project(vue), {
    typeChecker: { jsxTypecheck: true },
    ...global,
    formatter: { tabWidth: 2, singleQuote: true },
    entries: vue.entries,
  });
  assert.deepEqual(await project({ entries: [] }), {
    typeChecker: { jsxTypecheck: true },
    ...global,
    entries: [],
  });
  assert.deepEqual(await project({}), {
    typeChecker: { jsxTypecheck: true },
    ...global,
    entries,
  });
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
      vue: mergeVueConfigOptions(base, own),
    }),
    {
      typeChecker: { jsxTypecheck: true, enabled: false, strict: true },
      files: [],
      entries: [{ files: ["new/**"] }],
    },
  );
});

test("source envelopes retain composed canonical settings while retired extends and cycles refuse", async () => {
  let calls = 0;
  const wrapped = Object.assign(
    () => {
      throw new Error("factory must not run");
    },
    {
      [Symbol.for("@vizejs/vite-plugin/vite-plus/source")]: async () => {
        calls++;
        return {
          vue: mergeVueConfigOptions(
            { formatter: { tabWidth: 4 }, files: ["base/**"] },
            { formatter: { singleQuote: true } },
          ),
        };
      },
    },
  );
  assert.deepEqual(await resolveViteConfigExport(wrapped), {
    typeChecker: { jsxTypecheck: true },
    files: ["base/**"],
    formatter: { tabWidth: 4, singleQuote: true },
  });
  assert.equal(calls, 1);
  await assert.rejects(resolveViteConfigExport({ extends: wrapped }), /Custom config extends/);
  assert.equal(calls, 1, "retired inheritance never evaluates its source");
  const cyclic = {};
  cyclic[Symbol.for("@vizejs/vite-plugin/vite-plus/source")] = cyclic;
  await assert.rejects(resolveViteConfigExport(cyclic), /Circular Vue configuration source/);
});

test("canonical dynamic native maps preserve own prototype-shaped names without mutation", async () => {
  const vue = JSON.parse(
    '{"globalTypes":{"constructor":"boolean","__proto__":"number"},"linter":{"rules":{"__proto__":"warn"}}}',
  );
  const result = await projectViteConfigSource({
    vue: mergeVueConfigOptions({ globalTypes: { Other: "string" } }, vue),
  });
  assert.deepEqual(result, {
    typeChecker: { jsxTypecheck: true },
    globalTypes: JSON.parse('{"Other":"string","constructor":"boolean","__proto__":"number"}'),
    linter: { rules: JSON.parse('{"__proto__":"warn"}') },
  });
  assert.equal(Object.getPrototypeOf(result.globalTypes), Object.prototype);
});

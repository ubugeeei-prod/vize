import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveViteConfigExport } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

test("top-level formatter controls are fallbacks beneath Vue settings without mutating the source", async () => {
  const source = {
    fmt: {
      printWidth: 120,
      tabWidth: 4,
      semi: false,
      singleQuote: true,
      sortImports: false,
      ignorePatterns: ["ignored", "!ignored/keep.ts"],
      sortPackageJson: true,
      unknownHostOption: { value: true },
      vue: { printWidth: 110, singleQuote: false },
    },
    vue: { formatter: { printWidth: 90, tabWidth: 2 } },
  };
  const before = structuredClone(source);
  assert.deepEqual(await resolveViteConfigExport(source), {
    typeChecker: { jsxTypecheck: true },
    formatter: {
      printWidth: 90,
      tabWidth: 2,
      semi: false,
      singleQuote: false,
      sortImports: false,
      ignorePatterns: ["ignored", "!ignored/keep.ts"],
    },
  });
  assert.deepEqual(source, before);
  assert.deepEqual(await resolveViteConfigExport({ fmt: { printWidth: 120 } }), {
    typeChecker: { jsxTypecheck: true },
    formatter: { printWidth: 120 },
  });
  assert.deepEqual(await resolveViteConfigExport({ fmt: false }), {
    typeChecker: { jsxTypecheck: true },
  });
});

test("formatter ignore lists remain scoped and canonical empty lists replace the fallback", async () => {
  assert.deepEqual(
    await resolveViteConfigExport(
      {
        root: "app",
        fmt: { ignorePatterns: ["generated/**"] },
        vue: { ignores: ["global/**"], fmt: { ignorePatterns: [] } },
      },
      undefined,
      "/tmp/vize-formatter-root",
    ),
    {
      __vizeProjectRoot: "/tmp/vize-formatter-root/app",
      typeChecker: { jsxTypecheck: true },
      formatter: { ignorePatterns: [] },
      ignores: ["global/**"],
    },
  );
});

test("an Oxfmt-side exclusion of every Vue file keeps .vue with the native formatter", async () => {
  for (const pattern of ["**/*.vue", "*.vue", "**/**/*.vue"]) {
    assert.deepEqual(
      await resolveViteConfigExport({
        fmt: { semi: false, ignorePatterns: [pattern, "src/generated/**", "**/*.art.vue"] },
      }),
      {
        typeChecker: { jsxTypecheck: true },
        formatter: { semi: false, ignorePatterns: ["src/generated/**", "**/*.art.vue"] },
      },
      pattern,
    );
  }
  assert.deepEqual(
    await resolveViteConfigExport({
      fmt: { ignorePatterns: ["**/*.vue"], vue: { ignorePatterns: ["**/*.vue"] } },
    }),
    { typeChecker: { jsxTypecheck: true }, formatter: { ignorePatterns: ["**/*.vue"] } },
  );
  assert.deepEqual(
    await resolveViteConfigExport({
      fmt: { ignorePatterns: ["/*.vue", "src/*.vue", "!**/*.vue"] },
    }),
    {
      typeChecker: { jsxTypecheck: true },
      formatter: { ignorePatterns: ["/*.vue", "src/*.vue", "!**/*.vue"] },
    },
  );
});

test("malformed shared formatter controls refuse before source selection", async () => {
  for (const fmt of [
    { printWidth: "120" },
    { semi: 0 },
    { ignorePatterns: "ignored" },
    { ignorePatterns: ["ignored", false] },
    { vue: { tabWidth: "4" } },
  ]) {
    await assert.rejects(resolveViteConfigExport({ fmt }), /fmt\./);
  }
});

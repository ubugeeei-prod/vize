import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { validateTypecheckerOutput } from "../../tools/support/compat/fixtures/tool-matrix-typechecker.mjs";

const externalProgramFixture = new URL(
  "./fixtures/typechecker-external-program.json",
  import.meta.url,
);

function fixture() {
  return JSON.parse(readFileSync(externalProgramFixture, "utf8"));
}

function output(program: Record<string, unknown>) {
  return {
    files: [
      {
        file: "src/App.vue",
        diagnostics: ["error:1:1 [TS1] synthetic error"],
      },
    ],
    programs: [program],
    errorCount: 1,
    warningCount: 0,
    fileCount: 1,
  };
}

test("typechecker oracle accepts compiler options for tsconfig-backed programs", () => {
  validateTypecheckerOutput(
    { id: "tsconfig-program" },
    output({
      root: ".",
      tsconfig: "tsconfig.json",
      compilerOptions: {
        module: "ESNext",
        noUncheckedIndexedAccess: true,
        paths: { "@/*": ["./src/*"] },
      },
      files: ["src/App.vue"],
    }),
    1,
  );
});

test("ancestor declaration members preserve program metadata and checked-input evidence", () => {
  const payload = fixture();
  const original = structuredClone(payload);
  const inputs = payload.files.map((file: { file: string }) => file.file);
  const coverage = validateTypecheckerOutput({ id: "inertia" }, payload, 0, inputs, inputs);
  const relativeOnly = structuredClone(payload);
  relativeOnly.programs[0].files = inputs;
  assert.deepEqual(
    coverage,
    validateTypecheckerOutput({ id: "inertia" }, relativeOnly, 0, inputs, inputs),
  );
  assert.deepEqual(payload, original);
});

test("program member metadata accepts normalized POSIX, drive and UNC paths on every host", () => {
  for (const file of [
    "src/App.vue",
    "node_modules/vue/index.d.ts",
    "/home/runner/node_modules/vite/client.d.ts",
    "C:/repo/node_modules/vite/client.d.ts",
    "//server/share/node_modules/vite/client.d.ts",
  ]) {
    const payload = fixture();
    payload.programs[0].files = [file];
    assert.doesNotThrow(() => validateTypecheckerOutput({ id: "inertia" }, payload, 0));
  }
});

test("malformed program members remain rejected without weakening diagnostic path validation", () => {
  for (const file of [
    "",
    "../types.d.ts",
    "/repo/../types.d.ts",
    "C:/repo/./types.d.ts",
    "//server//types.d.ts",
    "./types.d.ts",
    "/repo//types.d.ts",
    "C:types.d.ts",
    "C:\\repo\\types.d.ts",
    "/repo/types.d.ts\0",
    "/",
    "C:/",
    "//server",
    null,
    42,
  ]) {
    const payload = fixture();
    payload.programs[0].files = [file];
    assert.throws(() => validateTypecheckerOutput({ id: "inertia" }, payload, 0), {
      message:
        "invalid typechecker JSON output: programs[0].files[0] must be a normalized program path",
    });
  }
  const payload = fixture();
  payload.files[0].file = "/repo/src/App.vue";
  assert.throws(() => validateTypecheckerOutput({ id: "inertia" }, payload, 0), {
    message: "invalid typechecker JSON output: files[0].file must be a normalized relative path",
  });
});

test("canonical Rust fixture oracle validates the same external-program regression and path controls", () => {
  const tool = new URL("../../tools/commands/fixtures/tool-matrix-report.rs", import.meta.url);
  const result = spawnSync("rust-script", ["--test", tool.pathname], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr + result.stdout);
});

test("typechecker oracle binds compiler options to tsconfig-backed programs", () => {
  assert.throws(
    () =>
      validateTypecheckerOutput(
        { id: "missing-options" },
        output({ root: ".", tsconfig: "tsconfig.json", files: ["src/App.vue"] }),
        1,
      ),
    /programs\[0\] keys must be compilerOptions, files, root, tsconfig/,
  );
  assert.throws(
    () =>
      validateTypecheckerOutput(
        { id: "orphan-options" },
        output({ root: ".", compilerOptions: {}, files: ["src/App.vue"] }),
        1,
      ),
    /programs\[0\] keys must be files, root/,
  );
  assert.throws(
    () =>
      validateTypecheckerOutput(
        { id: "malformed-options" },
        output({
          root: ".",
          tsconfig: "tsconfig.json",
          compilerOptions: [],
          files: ["src/App.vue"],
        }),
        1,
      ),
    /programs\[0\]\.compilerOptions must be an object/,
  );
});

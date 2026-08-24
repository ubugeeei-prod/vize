import assert from "node:assert/strict";
import { test } from "node:test";

import { validateTypecheckerOutput } from "../../tools/fixtures/tool-matrix-typechecker.mjs";

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

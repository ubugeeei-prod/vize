import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { emptyReport } from "../../fixtures/typechecker/vite-plus-relative-tsconfig/fixture.ts";
import { ProjectOracle } from "../../fixtures/typechecker/vite-plus-relative-tsconfig/runtime.ts";
import type {
  Receipt,
  Report,
} from "../../fixtures/typechecker/vite-plus-relative-tsconfig/runtime.ts";

const compilerOptions = {
  allowJs: true,
  checkJs: true,
  strict: true,
  target: "ES2022",
  module: "ESNext",
  moduleResolution: "bundler",
  noEmit: true,
};
const broken = "/** @type {string} */\nexport const message = 42;\n";
const allowedFile = "packages/allow/src/invalid.js";
const deniedFile = "packages/deny/src/ignored.js";
const solution = {
  files: [],
  references: [{ path: "./packages/allow" }, { path: "./packages/deny" }],
};

function report(valid: boolean): Report {
  return {
    files: [
      {
        file: allowedFile,
        diagnostics: valid
          ? []
          : ["error:2:14 [TS2322] Type 'number' is not assignable to type 'string'."],
      },
    ],
    programs: [
      {
        root: "packages/allow",
        tsconfig: "packages/allow/tsconfig.json",
        compilerOptions,
        files: [allowedFile],
      },
    ],
    errorCount: valid ? 0 : 1,
    warningCount: 0,
    fileCount: 1,
  };
}

function pair(direct: Receipt, task: Receipt, status: number, expected: Report, stderr = ""): void {
  for (const result of [direct, task]) {
    assert.equal(result.status, status, result.stderr || result.stdout);
    assert.equal(result.stderr, stderr);
    assert.deepEqual(result.report, expected);
    assert.equal(result.stdout, `${JSON.stringify(result.report, null, 2)}\n`);
  }
  assert.equal(task.stdout, direct.stdout);
  assert.equal(task.stderr, direct.stderr);
}

void test("owning allowJs exclusions retain explicit skip while real empty reference projects refuse", async () => {
  const oracle = await ProjectOracle.create("allowjs");
  try {
    oracle.write("tsconfig.app.json", JSON.stringify(solution));
    oracle.write(
      "packages/allow/tsconfig.json",
      JSON.stringify({ compilerOptions, include: ["src/**/*"] }),
    );
    oracle.write(
      "packages/deny/tsconfig.json",
      JSON.stringify({ compilerOptions: { allowJs: false }, include: ["src/**/*"] }),
    );
    oracle.write(allowedFile, broken);
    oracle.write(deniedFile, broken.replace("message", "ignored"));
    const direct = (label: string, inputs: string[] = []) =>
      oracle.direct(label, ["--no-config", "--tsconfig", "tsconfig.app.json", ...inputs]);

    pair(
      direct("denied-direct", [deniedFile]),
      await oracle.task("denied-task", [deniedFile]),
      0,
      emptyReport,
    );
    pair(
      direct("mixed-direct", [allowedFile, deniedFile]),
      await oracle.task("mixed-task", [allowedFile, deniedFile]),
      1,
      report(false),
    );
    pair(direct("default-direct"), await oracle.task("default-task"), 1, report(false));
    oracle.write("tsconfig.json", JSON.stringify(solution));
    pair(
      oracle.direct("discovered-direct", ["--no-config"]),
      await oracle.task("discovered-task", [], { typeChecker: {} }),
      1,
      report(false),
    );
    oracle.write(allowedFile, broken.replace("42", "'ok'"));
    pair(direct("repaired-direct"), await oracle.task("repaired-task"), 0, report(true));

    const refused = `Error: No supported source files were selected by TypeScript project \x60${path.join(oracle.root, "tsconfig.app.json")}\x60; no files were checked. Check the project's files/include/exclude and Vize ignores.\n`;
    pair(
      direct("unmatched-direct", ["packages/deny/src/missing.js"]),
      await oracle.task("unmatched-task", ["packages/deny/src/missing.js"]),
      2,
      emptyReport,
      refused,
    );
    oracle.write(
      "packages/allow/tsconfig.json",
      JSON.stringify({
        compilerOptions: { ...compilerOptions, allowJs: false },
        include: ["src/**/*"],
      }),
    );
    pair(direct("empty-direct"), await oracle.task("empty-task"), 2, emptyReport, refused);
  } finally {
    oracle.finish();
  }
});

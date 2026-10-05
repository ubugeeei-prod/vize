import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  diagnostic,
  emptyReport,
  originalInputs,
} from "../../fixtures/typechecker/vite-plus-relative-tsconfig/fixture.ts";
import { ProjectOracle } from "../../fixtures/typechecker/vite-plus-relative-tsconfig/runtime.ts";
import type {
  Receipt,
  Report,
} from "../../fixtures/typechecker/vite-plus-relative-tsconfig/runtime.ts";

const original = originalInputs().inputs;
const compilerOptions = JSON.parse(original["tsconfig.app.json"]).compilerOptions as Record<
  string,
  unknown
>;
const configured = { typeChecker: { tsconfig: "tsconfig.app.json" } };

function expectedReport(
  valid: boolean,
  root = ".",
  tsconfig = "tsconfig.app.json",
  file = "src/Counter.vue",
): Report {
  return {
    files: [{ file, diagnostics: valid ? [] : [diagnostic] }],
    programs: [{ root, tsconfig, compilerOptions, files: [file] }],
    errorCount: valid ? 0 : 1,
    warningCount: 0,
    fileCount: 1,
  };
}

function sameWholeReport(
  actual: Receipt,
  expected: Receipt,
  valid: boolean,
  report = expectedReport(valid),
): void {
  assert.equal(actual.status, valid ? 0 : 1, actual.stderr || actual.stdout);
  assert.equal(expected.status, actual.status, expected.stderr || expected.stdout);
  assert.equal(actual.stderr, "");
  assert.equal(expected.stderr, "");
  assert.equal(
    actual.stdout,
    expected.stdout,
    "the complete CLI JSON stream must match byte for byte",
  );
  assert.deepEqual(actual.report, expected.report);
  assert.deepEqual(actual.report, report);
}

function refused(receipt: Receipt, project: string, missing = false): void {
  assert.equal(receipt.status, 2, receipt.stderr || receipt.stdout);
  assert.deepEqual(receipt.report, emptyReport);
  assert.equal(
    receipt.stderr,
    missing
      ? `Error: TypeScript config file not found: \x60${project}\x60; no files were checked.\n`
      : `Error: No supported source files were selected by TypeScript project \x60${project}\x60; no files were checked. Check the project's files/include/exclude and Vize ignores.\n`,
  );
}

void test("original Vite+ relative project checks the whole Counter input through invalid→repair→invalid", async () => {
  const oracle = await ProjectOracle.create("original");
  try {
    for (const [id, valid] of [
      ["original", false],
      ["repaired", true],
      ["restored", false],
    ] as const) {
      oracle.write(
        "src/Counter.vue",
        valid
          ? original["src/Counter.vue"].replace('"not a number"', "1")
          : original["src/Counter.vue"],
      );
      const direct = oracle.direct(`${id}-direct`, [
        "--no-config",
        "--tsconfig",
        "tsconfig.app.json",
      ]);
      const task = await oracle.task(`${id}-task`);
      sameWholeReport(task, direct, valid);
    }
    const direct = oracle.direct("virtual-direct", [
      "--no-config",
      "--tsconfig",
      "tsconfig.app.json",
      "--show-virtual-ts",
    ]);
    const task = await oracle.task("virtual-task", ["--show-virtual-ts"]);
    assert.equal(task.status, 1);
    assert.equal(direct.status, 1);
    assert.equal(task.stderr, direct.stderr, "complete debug dumps must match byte for byte");
    assert(task.stderr.startsWith("\n=== __vize_helpers.d.ts ===\n"));
    assert.deepEqual(task.stderr.match(/^=== .* ===$/gm), [
      "=== __vize_helpers.d.ts ===",
      `=== ${path.join(oracle.root, "src/Counter.vue")} ===`,
    ]);
    assert.equal(task.stdout, direct.stdout);
    assert.deepEqual(
      task.report,
      direct.report,
      "complete generated TypeScript bytes must match too",
    );
    assert(task.report);
    assert.equal(typeof task.report.files[0].virtualTs, "string");
    assert(task.stderr.endsWith(`${task.report.files[0].virtualTs}\n`));
    assert(task.report.files[0].virtualTs?.includes('const count: number = "not a number";'));
    const withoutVirtual = {
      ...task.report,
      files: task.report.files.map(({ virtualTs: _virtual, ...file }) => file),
    };
    assert.deepEqual(withoutVirtual, expectedReport(false));
    assert.equal(
      fs.readFileSync(path.join(oracle.root, "src/Counter.vue"), "utf8"),
      original["src/Counter.vue"],
    );
  } finally {
    oracle.finish();
  }
});

void test("configured project paths preserve absolute, nested, config-directory, and explicit CLI override semantics", async () => {
  const oracle = await ProjectOracle.create("paths");
  try {
    const direct = oracle.direct("direct", ["--no-config", "--tsconfig", "tsconfig.app.json"]);
    sameWholeReport(
      await oracle.task("absolute", [], {
        typeChecker: { tsconfig: path.join(oracle.root, "tsconfig.app.json") },
      }),
      direct,
      false,
    );
    oracle.write(
      "config/vize.json",
      JSON.stringify({ typeChecker: { tsconfig: "../tsconfig.app.json" } }),
    );
    sameWholeReport(
      oracle.direct("config-directory", ["--config", "config/vize.json"]),
      direct,
      false,
    );
    sameWholeReport(
      await oracle.task("explicit-override", ["--tsconfig", "tsconfig.app.json"], {
        typeChecker: { tsconfig: "missing.json" },
      }),
      direct,
      false,
    );
    const nested = { compilerOptions, include: ["../src/**/*.vue"] };
    oracle.write("config/tsconfig.json", JSON.stringify(nested));
    const nestedDirect = oracle.direct("nested-direct", [
      "--no-config",
      "--tsconfig",
      "config/tsconfig.json",
    ]);
    const nestedTask = await oracle.task("nested-task", [], {
      typeChecker: { tsconfig: "config/tsconfig.json" },
    });
    sameWholeReport(
      nestedTask,
      nestedDirect,
      false,
      expectedReport(false, "config", "config/tsconfig.json"),
    );
  } finally {
    oracle.finish();
  }
});

void test("missing and zero-workload selected projects fail without inventing diagnostic success", async () => {
  const oracle = await ProjectOracle.create("refusal");
  try {
    for (const name of ["missing.json", "missing/tsconfig.json", "directory"]) {
      if (name === "directory") fs.mkdirSync(path.join(oracle.root, name));
      const selected = path.join(oracle.root, name);
      refused(oracle.direct(`${name}-direct`, ["--no-config", "--tsconfig", name]), selected, true);
      refused(
        await oracle.task(`${name}-task`, [], { typeChecker: { tsconfig: name } }),
        selected,
        true,
      );
    }
    const emptyProjects = [
      { compilerOptions, include: ["missing/**/*.vue"] },
      { compilerOptions, files: [] },
      { compilerOptions, include: ["src/**/*.vue"], exclude: ["src/**"] },
    ];
    for (const [index, config] of emptyProjects.entries()) {
      oracle.write("tsconfig.app.json", JSON.stringify(config));
      const selected = path.join(oracle.root, "tsconfig.app.json");
      refused(
        oracle.direct(`empty-${index}-direct`, ["--no-config", "--tsconfig", "tsconfig.app.json"]),
        selected,
      );
      refused(await oracle.task(`empty-${index}-task`), selected);
    }
    oracle.write("tsconfig.app.json", original["tsconfig.app.json"]);
    refused(
      oracle.direct("missing-with-authored-input", [
        "--no-config",
        "--tsconfig",
        "missing.json",
        "src/Counter.vue",
      ]),
      path.join(oracle.root, "missing.json"),
      true,
    );
    refused(
      oracle.direct("selected-unmatched", [
        "--no-config",
        "--tsconfig",
        "tsconfig.app.json",
        "does-not-exist.vue",
      ]),
      path.join(oracle.root, "tsconfig.app.json"),
    );
    refused(
      await oracle.task("ignored-project", [], { ...configured, ignores: ["src/**"] }),
      path.join(oracle.root, "tsconfig.app.json"),
    );
    const missingAbsolute = path.join(oracle.root, "absolute-missing.json");
    refused(
      await oracle.task("absolute-missing", [], { typeChecker: { tsconfig: missingAbsolute } }),
      missingAbsolute,
      true,
    );
    const text = oracle.direct(
      "missing-text",
      ["--no-config", "--tsconfig", "missing.json"],
      false,
    );
    assert.equal(text.status, 2);
    assert.equal(text.stdout, "");
    assert.equal(
      text.stderr,
      `Error: TypeScript config file not found: \x60${path.join(oracle.root, "missing.json")}\x60; no files were checked.\n`,
    );
    oracle.write("tsconfig.json", JSON.stringify({ files: [] }));
    refused(
      oracle.direct("auto-discovered-empty", ["--no-config"]),
      path.join(oracle.root, "tsconfig.json"),
    );
  } finally {
    oracle.finish();
  }
});

void test("unconfigured no-match and intentionally disabled checking keep the existing empty contract", async () => {
  const oracle = await ProjectOracle.create("legacy-empty");
  try {
    oracle.write("vize.config.json", JSON.stringify({ typeChecker: { tsconfig: "missing.json" } }));
    const unmatched = oracle.direct("unconfigured-unmatched", [
      "--no-config",
      "does-not-exist.vue",
    ]);
    assert.equal(unmatched.status, 0);
    assert.equal(unmatched.stderr, "");
    assert.deepEqual(unmatched.report, emptyReport);
    fs.rmSync(path.join(oracle.root, "src"), { recursive: true });
    fs.rmSync(path.join(oracle.root, "vite.config.mts"));
    fs.rmSync(path.join(oracle.root, "tsconfig.app.json"));
    const empty = oracle.direct("unconfigured-empty", ["--no-config"]);
    assert.equal(empty.status, 0);
    assert.equal(empty.stderr, "");
    assert.deepEqual(empty.report, emptyReport);
    const disabled = await oracle.task("disabled", [], {
      typeChecker: { enabled: false, tsconfig: "missing.json" },
    });
    assert.equal(disabled.status, 0);
    assert.equal(disabled.stdout, "");
    assert.equal(
      disabled.stderr,
      "[vize] Skipping check because typeChecker.enabled is false in vize.config.\n",
    );
    assert.equal(disabled.report, null);
    assert.deepEqual(await oracle.disabledFrontend(), { status: 0, launches: 0 });
  } finally {
    oracle.finish();
  }
});

void test("a files-empty solution with a nonempty referenced child is checked as a real program", async () => {
  const oracle = await ProjectOracle.create("references");
  try {
    oracle.write("app/src/Counter.vue", original["src/Counter.vue"]);
    oracle.write(
      "app/tsconfig.json",
      JSON.stringify({ compilerOptions, include: ["src/**/*.vue"] }),
    );
    oracle.write(
      "tsconfig.app.json",
      JSON.stringify({ files: [], references: [{ path: "./app" }] }),
    );
    const direct = oracle.direct("referenced-direct", [
      "--no-config",
      "--tsconfig",
      "tsconfig.app.json",
    ]);
    const task = await oracle.task("referenced-task");
    sameWholeReport(
      task,
      direct,
      false,
      expectedReport(false, "app", "app/tsconfig.json", "app/src/Counter.vue"),
    );
    oracle.write("app/src/Counter.vue", original["src/Counter.vue"].replace('"not a number"', "1"));
    sameWholeReport(
      await oracle.task("referenced-repair-task"),
      oracle.direct("referenced-repair-direct", ["--no-config", "--tsconfig", "tsconfig.app.json"]),
      true,
      expectedReport(true, "app", "app/tsconfig.json", "app/src/Counter.vue"),
    );
  } finally {
    oracle.finish();
  }
});

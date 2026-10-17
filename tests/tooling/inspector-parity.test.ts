import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, renameSync, rmSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import {
  assertInspectorFileCoverage,
  inspectorReportDirectory,
} from "../_helpers/inspector-parity.ts";

test("inspector reports use the shared artifact directory by default", () => {
  const outputDirectory = inspectorReportDirectory("Example / Application");

  const expectedSuffix = path.join(".vize", "artifacts", "inspect", "Example-Application");
  assert.ok(
    outputDirectory.endsWith(expectedSuffix),
    `Expected path to end with "${expectedSuffix}", but got "${outputDirectory}"`,
  );
});

test("inspector reports preserve an explicit output directory", () => {
  assert.equal(
    inspectorReportDirectory("Example", "/tmp/inspection-output"),
    "/tmp/inspection-output",
  );
});

test("inspector coverage rejects omitted, duplicate or substituted original inputs", (t) => {
  const cwd = createRepository(t);
  mkdirSync(path.join(cwd, "app", "nested"), { recursive: true });
  for (const file of ["app/App.vue", "app/nested/Child.vue", "app/ignored.ts"]) {
    writeFileSync(path.join(cwd, file), "original source\n");
  }
  const check = { cwd, patterns: ["app/**/*.vue", "app/App.vue"] };
  const report = (paths: string[], fileCount = paths.length) => ({
    summary: { fileCount },
    files: paths.map((file) => ({ path: file })),
  });
  const inputs = ["app/App.vue", "app/nested/Child.vue"];
  const identity = {
    fileCount: 2,
    pathsSha256: createHash("sha256").update("app/App.vue\napp/nested/Child.vue\n").digest("hex"),
  };
  assertInspectorFileCoverage(check, report(inputs.toReversed()), identity);
  for (const paths of [
    [inputs[0]],
    [inputs[0], inputs[0]],
    [inputs[0], "app/ignored.ts"],
    [...inputs, "app/extra.vue"],
    [],
  ]) {
    assert.throws(
      () => assertInspectorFileCoverage(check, report(paths, 2), identity),
      /every original source exactly once/,
    );
  }
  assert.throws(
    () => assertInspectorFileCoverage(check, report(inputs, 1), identity),
    /source count/,
  );
  writeFileSync(path.join(cwd, "app", "Added.vue"), "new authored source\n");
  assert.throws(
    () => assertInspectorFileCoverage(check, report([...inputs, "app/Added.vue"]), identity),
    /pinned original source count/,
  );
  unlinkSync(path.join(cwd, "app", "Added.vue"));
  assert.throws(
    () => assertInspectorFileCoverage({ cwd, patterns: ["absent/**/*.vue"] }, report([]), identity),
    /must contain original source inputs/,
  );
  renameSync(path.join(cwd, inputs[1]), path.join(cwd, "app/nested/Renamed.vue"));
  assert.throws(
    () =>
      assertInspectorFileCoverage(check, report([inputs[0], "app/nested/Renamed.vue"]), identity),
    /pinned original source paths/,
  );
  unlinkSync(path.join(cwd, "app/nested/Renamed.vue"));
  assert.throws(
    () => assertInspectorFileCoverage(check, report([inputs[0]]), identity),
    /pinned original source count/,
  );
});

test("recursive inspector inputs honor authored ignores without consulting the Git index", (t) => {
  const cwd = createRepository(t);
  mkdirSync(path.join(cwd, "app", "generated"), { recursive: true });
  mkdirSync(path.join(cwd, "app", "nested"));
  writeFileSync(path.join(cwd, ".gitignore"), "/app/generated/\n");
  writeFileSync(path.join(cwd, "app", "nested", ".gitignore"), "*.vue\n!Child.vue\n");
  for (const file of ["app/App.vue", "app/nested/Child.vue", "app/nested/Ignored.vue"]) {
    writeFileSync(path.join(cwd, file), "<template />\n");
  }
  for (let index = 0; index < 51; index++) {
    writeFileSync(path.join(cwd, "app", "generated", `Generated ${index}.vue`), "generated\n");
  }
  execFileSync("git", ["add", "--force", "app/generated/Generated 0.vue"], { cwd });
  const original = ["app/App.vue", "app/nested/Child.vue"];
  for (const patterns of [["app/**/*.vue"], ["app"], ["**/*.vue"], ["./**/*.vue"]]) {
    assertCoverage(cwd, patterns, original);
  }
  // Explicit inputs override the walk's ignores, while generic globs do not
  // consult ignores at all, matching the native Inspector's separate branches.
  assertCoverage(
    cwd,
    ["app/**/*.vue", "app/generated/Generated 0.vue"],
    [...original, "app/generated/Generated 0.vue"],
  );
  assertCoverage(cwd, ["app/nested/*.vue"], ["app/nested/Child.vue", "app/nested/Ignored.vue"]);
});

test("inspector coverage fails when Git cannot run or evaluate the authored ignores", (t) => {
  const cwd = createRepository(t);
  mkdirSync(path.join(cwd, "app"));
  writeFileSync(path.join(cwd, "app", "App.vue"), "<template />\n");
  rmSync(path.join(cwd, ".git"), { recursive: true });
  writeFileSync(path.join(cwd, ".git"), "gitdir: missing-repository\n");
  assert.throws(
    () => assertCoverage(cwd, ["app/**/*.vue"], ["app/App.vue"]),
    /cannot determine inspector Git ignores \(exit 128\)/,
  );
  const originalPath = process.env.PATH;
  try {
    process.env.PATH = path.join(cwd, "missing-executables");
    assert.throws(() => assertCoverage(cwd, ["app/**/*.vue"], ["app/App.vue"]), {
      code: "ENOENT",
      syscall: "spawnSync git",
    });
  } finally {
    if (originalPath === undefined) delete process.env.PATH;
    else process.env.PATH = originalPath;
  }
});

function createRepository(t: TestContext): string {
  const cwd = mkdtempSync(path.join(tmpdir(), "inspector-inputs-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  execFileSync("git", ["init", "--quiet", cwd]);
  return cwd;
}

function assertCoverage(cwd: string, patterns: string[], paths: string[]): void {
  const sorted = paths.toSorted();
  assertInspectorFileCoverage(
    { cwd, patterns },
    { summary: { fileCount: paths.length }, files: paths.map((file) => ({ path: file })) },
    {
      fileCount: paths.length,
      pathsSha256: createHash("sha256")
        .update(`${sorted.join("\n")}\n`)
        .digest("hex"),
    },
  );
}

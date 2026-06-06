import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/**
 * Resolves how to launch the `vize` CLI, mirroring cli-check-contract.test.ts:
 * a prebuilt binary wins, otherwise fall back to cargo so fresh checkouts work.
 */
function resolveVizeCommand(): { command: string; prefix: string[] } {
  const candidates = [
    path.join(root, "target/ci/vize"),
    path.join(root, "target/release/vize"),
    path.join(root, "target/debug/vize"),
    "vize",
  ];
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["--version"], { cwd: root, encoding: "utf8" });
    if (probe.status === 0) {
      return { command: candidate, prefix: [] };
    }
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}

const VIZE = resolveVizeCommand();

// A path that does not exist, used to prove that the collection short-circuits
// happen BEFORE the type-checker (corsa) binary is ever discovered/spawned.
const NONEXISTENT_CORSA = "/nonexistent";

/**
 * Resolves a real type-checker binary for the one case that genuinely runs
 * corsa. The spec calls for node_modules/.bin/{corsa,tsgo}; tsgo is the
 * committed dev dependency. Returns null when none is runnable so the case can
 * skip rather than fail in a checker-less environment.
 */
function resolveCheckerPath(): string | null {
  const candidates = [
    path.join(root, "node_modules/.bin/corsa"),
    path.join(root, "node_modules/.bin/tsgo"),
  ];
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["--version"], { cwd: root, encoding: "utf8" });
    if (!probe.error && probe.status === 0) {
      return candidate;
    }
  }
  return null;
}

type CheckResult = { status: number | null; stdout: string; stderr: string };

function runCheck(args: string[], cwd: string): CheckResult {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "check", ...args], {
    cwd,
    encoding: "utf8",
  });
  if (result.error) {
    throw result.error;
  }
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

// The workspace must live outside the repository tree: `vize check` with no
// explicit inputs walks up looking for a tsconfig.json, and a repo-local temp
// dir would inherit the workspace tsconfig and start type-checking the repo.
function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-collection-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

test("empty collection emits clean JSON and exits 0 without invoking corsa", () => {
  withWorkspace((dir) => {
    // No input files at all: collection is empty and corsa is never reached, so
    // a deliberately broken --corsa-path must not affect the result.
    const noInputs = runCheck(["--format", "json", "--corsa-path", NONEXISTENT_CORSA], dir);
    assert.equal(noInputs.status, 0, `${noInputs.stdout}\n${noInputs.stderr}`);
    const noInputsParsed = JSON.parse(noInputs.stdout);
    assert.deepEqual(noInputsParsed, {
      files: [],
      errorCount: 0,
      warningCount: 0,
      fileCount: 0,
    });
    assert.ok(!("declarations" in noInputsParsed), "declarations must be absent");

    // An explicit pattern that matches nothing yields the same empty report.
    const noMatch = runCheck(
      ["nope.vue", "--format", "json", "--corsa-path", NONEXISTENT_CORSA],
      dir,
    );
    assert.equal(noMatch.status, 0, `${noMatch.stdout}\n${noMatch.stderr}`);
    const noMatchParsed = JSON.parse(noMatch.stdout);
    assert.deepEqual(noMatchParsed, {
      files: [],
      errorCount: 0,
      warningCount: 0,
      fileCount: 0,
    });
    assert.ok(!("declarations" in noMatchParsed), "declarations must be absent");
  });
});

test("no-match text run prints the notice to stderr, leaves stdout empty, exits 0", () => {
  withWorkspace((dir) => {
    const result = runCheck(["zzz.vue", "--corsa-path", NONEXISTENT_CORSA], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    // The diagnostic stream separation is the load-bearing property here:
    // nothing on stdout, the human notice on stderr.
    assert.equal(result.stdout.trim(), "");
    assert.match(result.stderr, /No Vue or TypeScript files found matching inputs/);
    assert.ok(
      result.stderr.includes("zzz.vue"),
      `stderr should name the pattern: ${result.stderr}`,
    );
  });
});

test("unsupported file extensions (.js) collect nothing", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "skip.js"), "console.log(1)\n", "utf8");
    const result = runCheck(
      ["skip.js", "--format", "json", "--corsa-path", NONEXISTENT_CORSA],
      dir,
    );
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    assert.deepEqual(parsed, {
      files: [],
      errorCount: 0,
      warningCount: 0,
      fileCount: 0,
    });
  });
});

// Kept LAST in the file: this case needs a real, runnable type checker.
const checkerPath = resolveCheckerPath();
test(
  "tsconfig-driven default collection honors include/exclude through the CLI",
  { skip: checkerPath === null ? "no runnable corsa/tsgo found" : false },
  () => {
    withWorkspace((dir) => {
      fs.mkdirSync(path.join(dir, "src/generated"), { recursive: true });
      fs.writeFileSync(
        path.join(dir, "src/App.vue"),
        '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><p>{{ x }}</p></template>\n',
        "utf8",
      );
      fs.writeFileSync(
        path.join(dir, "src/main.ts"),
        'export const greet = (): string => "hi"\n',
        "utf8",
      );
      fs.writeFileSync(
        path.join(dir, "src/generated/skip.ts"),
        "export const skipped = 1\n",
        "utf8",
      );
      fs.writeFileSync(path.join(dir, "vite.config.ts"), "export default {}\n", "utf8");
      fs.writeFileSync(
        path.join(dir, "tsconfig.json"),
        JSON.stringify(
          {
            compilerOptions: { strict: true, module: "esnext", target: "esnext" },
            include: ["src/**/*.ts", "src/**/*.vue"],
            exclude: ["src/generated"],
          },
          null,
          2,
        ),
        "utf8",
      );

      // No explicit PATTERNS: collection is driven entirely by tsconfig
      // include/exclude.
      const result = runCheck(["--format", "json", "--corsa-path", checkerPath as string], dir);
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      const parsed = JSON.parse(result.stdout);
      const collected = parsed.files.map((f: { file: string }) => f.file).sort();
      assert.deepEqual(collected, ["src/App.vue", "src/main.ts"]);
      // Excluded directory and a file outside the include globs must be absent.
      assert.ok(!collected.includes("src/generated/skip.ts"));
      assert.ok(!collected.includes("vite.config.ts"));
      assert.equal(parsed.fileCount, 2);
    });
  },
);

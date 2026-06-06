import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/**
 * Resolves how to launch the `vize` CLI, mirroring the check-contract launcher:
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

/**
 * `--format json` runs the inputs through the Corsa/tsgo type-checker, so these
 * cases require a discoverable checker. Returns null when neither is installed
 * so the suite skips gracefully instead of failing (mirrors production-readiness).
 */
function resolveCheckerPath(): string | null {
  const candidates = [
    path.join(root, "node_modules/.bin/corsa"),
    path.join(root, "node_modules/.bin/tsgo"),
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

const CHECKER = resolveCheckerPath();

type CheckResult = { status: number | null; stdout: string; stderr: string };

function runCheck(args: string[], cwd: string): CheckResult {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "check", ...args], {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) {
    throw result.error;
  }
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

// Type-checking walks up looking for a tsconfig.json, so the workspace must live
// outside the repository tree (a repo-local temp dir would inherit the workspace
// config). os.tmpdir() keeps every fixture isolated.
function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-json-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

type CheckJson = {
  files: Array<{ file: string; virtualTs: string; diagnostics: string[] }>;
  errorCount: number;
  warningCount: number;
  fileCount: number;
};

function parseJson(result: CheckResult): CheckJson {
  return JSON.parse(result.stdout) as CheckJson;
}

// A trivial canonical TS2322: `number` assigned to a `string` binding. Used only
// to exercise the JSON envelope when diagnostics are present; the cases below
// never assert the checker's message text, only the stable JSON shape.
const BAD_TS = "export const x: string = 123;";

test(
  "vize check --format json has a stable top-level shape and key names",
  {
    skip: CHECKER == null ? "no corsa/tsgo checker discoverable" : false,
  },
  () => {
    withWorkspace((dir) => {
      fs.writeFileSync(path.join(dir, "bad.ts"), BAD_TS, "utf8");
      const result = runCheck(
        ["bad.ts", "--format", "json", "--corsa-path", CHECKER as string],
        dir,
      );
      assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);

      const parsed = parseJson(result);
      assert.deepEqual(
        Object.keys(parsed).sort(),
        ["errorCount", "fileCount", "files", "warningCount"],
        "top-level keys should be exactly the documented camelCase envelope",
      );
      // `--declaration` is absent, so the emitter must not surface a declarations key.
      assert.equal("declarations" in parsed, false, "no declarations key without --declaration");

      assert.ok(Array.isArray(parsed.files), "files should be an array");
      for (const entry of parsed.files) {
        assert.deepEqual(
          Object.keys(entry).sort(),
          ["diagnostics", "file", "virtualTs"],
          "each file entry should expose exactly file/virtualTs/diagnostics",
        );
        assert.equal(typeof entry.file, "string");
        assert.equal(typeof entry.virtualTs, "string");
        assert.ok(Array.isArray(entry.diagnostics));
      }

      assert.equal(typeof parsed.errorCount, "number");
      assert.equal(typeof parsed.warningCount, "number");
      assert.equal(typeof parsed.fileCount, "number");
    });
  },
);

test(
  "vize check --format json reports files cwd-relative, '/'-separated and sorted",
  {
    skip: CHECKER == null ? "no corsa/tsgo checker discoverable" : false,
  },
  () => {
    withWorkspace((dir) => {
      fs.mkdirSync(path.join(dir, "src"));
      fs.writeFileSync(
        path.join(dir, "src/Good.vue"),
        '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ x }}</div></template>\n',
        "utf8",
      );
      fs.writeFileSync(
        path.join(dir, "src/Bad.vue"),
        '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ unclosed </div></template>\n',
        "utf8",
      );

      // Pass Good before Bad on the command line; the report must come back sorted.
      const result = runCheck(
        ["src/Good.vue", "src/Bad.vue", "--format", "json", "--corsa-path", CHECKER as string],
        dir,
      );
      const parsed = parseJson(result);

      const reported = parsed.files.map((f) => f.file);
      assert.deepEqual(
        reported,
        ["src/Bad.vue", "src/Good.vue"],
        "files should be reported cwd-relative and sorted ascending",
      );
      assert.equal(parsed.fileCount, parsed.files.length);
      assert.equal(parsed.fileCount, 2);
      for (const file of reported) {
        assert.equal(file.includes("\\"), false, `path should use '/' separators: ${file}`);
      }
    });
  },
);

test(
  "vize check --format json reports only the requested subset of files",
  {
    skip: CHECKER == null ? "no corsa/tsgo checker discoverable" : false,
  },
  () => {
    withWorkspace((dir) => {
      fs.mkdirSync(path.join(dir, "src"));
      fs.writeFileSync(
        path.join(dir, "src/Good.vue"),
        '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ x }}</div></template>\n',
        "utf8",
      );
      // Sibling with an unterminated interpolation: it must never appear in the
      // report when only Good.vue is checked.
      fs.writeFileSync(
        path.join(dir, "src/Bad.vue"),
        '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ unclosed </div></template>\n',
        "utf8",
      );

      const result = runCheck(
        ["src/Good.vue", "--format", "json", "--corsa-path", CHECKER as string],
        dir,
      );
      const parsed = parseJson(result);

      assert.equal(parsed.files.length, 1, "only the explicitly requested file should be reported");
      assert.equal(parsed.files[0]?.file, "src/Good.vue");
      assert.ok(
        parsed.files.every((f) => f.file !== "src/Bad.vue"),
        "the unrequested sibling must not appear in the report",
      );
    });
  },
);

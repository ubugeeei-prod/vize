import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { expectsLintReport, getLintTargets } from "./args.ts";
import { verifyOxlintCliEntrypoint } from "./oxlint.ts";

const packageDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const cliEntry = path.join(packageDir, "dist", "cli.mjs");

/** The vite-plus LSP-only wrapper, reduced to the behavior that matters here. */
const LSP_ONLY_SHIM = `console.error("This oxlint wrapper is for IDE extension use only (--lsp mode).");
process.exit(0);
`;

const SILENT_SHIM = "process.exit(0);\n";

/** Answers the handshake like real oxlint but swallows every lint run. */
const HANDSHAKE_ONLY_SHIM = `if (process.argv.includes("--version")) {
  console.log("Version: 9.9.9");
}
process.exit(0);
`;

const REAL_LOOKING_OXLINT = 'console.log("Version: 1.78.0");\nprocess.exit(0);\n';

function withTempWorkspace<T>(run: (root: string) => T): T {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-oxlint-shim-"));
  try {
    return run(root);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

function writeFakeOxlintEntrypoint(root: string, source: string): string {
  const entrypoint = path.join(root, "node_modules", "oxlint", "bin", "oxlint");
  fs.mkdirSync(path.dirname(entrypoint), { recursive: true });
  fs.writeFileSync(entrypoint, source);
  return entrypoint;
}

function writeVueFixture(root: string): void {
  fs.writeFileSync(
    path.join(root, "App.vue"),
    `<script setup lang="ts">
const items = [1]
</script>
<template>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
`,
  );
}

function writeScriptlessVueFixture(root: string): void {
  fs.writeFileSync(
    path.join(root, "Scriptless.vue"),
    `<template>
  <div>{{ message }}</div>
</template>
`,
  );
}

function runOxlintVize(root: string, args: readonly string[]) {
  const result = spawnSync(process.execPath, [cliEntry, ...args], {
    cwd: root,
    encoding: "utf8",
    stdio: "pipe",
  });
  if (result.error) {
    throw result.error;
  }

  return {
    exitCode: result.status ?? 1,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

void test("verifyOxlintCliEntrypoint accepts a binary that answers the --version handshake", () => {
  withTempWorkspace((root) => {
    const entrypoint = writeFakeOxlintEntrypoint(root, REAL_LOOKING_OXLINT);
    verifyOxlintCliEntrypoint(process.execPath, entrypoint);
  });
});

void test("verifyOxlintCliEntrypoint rejects the LSP-only wrapper shim", () => {
  withTempWorkspace((root) => {
    const entrypoint = writeFakeOxlintEntrypoint(root, LSP_ONLY_SHIM);
    assert.throws(
      () => {
        verifyOxlintCliEntrypoint(process.execPath, entrypoint);
      },
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.match(error.message, /--version/u);
        assert.ok(error.message.includes(entrypoint), "the diagnostic should name the shim path");
        return true;
      },
    );
  });
});

void test("verifyOxlintCliEntrypoint rejects a shim that exits 0 with no output", () => {
  withTempWorkspace((root) => {
    const entrypoint = writeFakeOxlintEntrypoint(root, SILENT_SHIM);
    assert.throws(() => {
      verifyOxlintCliEntrypoint(process.execPath, entrypoint);
    }, /--version/u);
  });
});

void test("expectsLintReport recognizes the formats that always produce output", () => {
  assert.equal(expectsLintReport([]), false);
  assert.equal(expectsLintReport(["src"]), false);
  assert.equal(expectsLintReport(["-f", "json", "src"]), true);
  assert.equal(expectsLintReport(["--format", "junit"]), true);
  assert.equal(expectsLintReport(["--format=github"]), false);
  assert.equal(expectsLintReport(["-f", "stylish", "src"]), false);
  assert.equal(expectsLintReport(["-f", "unix"]), false);
  // After `--` everything is a lint target, not an option.
  assert.equal(expectsLintReport(["--", "-f", "json"]), false);
});

void test("getLintTargets treats fix flags as booleans", () => {
  assert.deepEqual(getLintTargets(["--fix", "src/App.vue"]), ["src/App.vue"]);
  assert.deepEqual(getLintTargets(["--fix-suggestions", "src/App.vue"]), ["src/App.vue"]);
});

void test("scriptless workaround warns for fix flags whose edits are not copied back", () => {
  for (const flag of ["--fix", "--fix-suggestions"] as const) {
    withTempWorkspace((root) => {
      writeFakeOxlintEntrypoint(root, HANDSHAKE_ONLY_SHIM);
      writeScriptlessVueFixture(root);

      const run = runOxlintVize(root, [flag, "Scriptless.vue"]);

      assert.equal(run.exitCode, 0);
      assert.match(run.stderr, /fixes are not applied back to original files/u);
    });
  }
});

void test("a workspace whose oxlint is a non-lint wrapper shim fails the run closed", () => {
  withTempWorkspace((root) => {
    const entrypoint = writeFakeOxlintEntrypoint(root, LSP_ONLY_SHIM);
    writeVueFixture(root);

    const run = runOxlintVize(root, ["App.vue"]);
    assert.notEqual(
      run.exitCode,
      0,
      "a shim that lints nothing must not be reported as a clean run",
    );
    assert.match(run.stderr, /--version/u);
    assert.ok(run.stderr.includes(entrypoint), "the diagnostic should name the shim path");
  });
});

void test("a child that answers the handshake but swallows the report fails closed", () => {
  withTempWorkspace((root) => {
    writeFakeOxlintEntrypoint(root, HANDSHAKE_ONLY_SHIM);
    writeVueFixture(root);

    const jsonRun = runOxlintVize(root, ["-f", "json", "App.vue"]);
    assert.notEqual(
      jsonRun.exitCode,
      0,
      "an empty run must not pass when the requested format always produces a report",
    );
    assert.match(jsonRun.stderr, /no report|produced no/iu);

    // The auto format legitimately prints nothing on a clean run, so a silent
    // exit-0 child stays trusted once it has answered the handshake.
    const autoRun = runOxlintVize(root, ["App.vue"]);
    assert.equal(autoRun.exitCode, 0);

    const githubRun = runOxlintVize(root, ["--format=github", "App.vue"]);
    assert.equal(githubRun.exitCode, 0);
    assert.equal(githubRun.stderr, "");
  });
});

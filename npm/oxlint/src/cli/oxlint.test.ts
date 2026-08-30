import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { expectsLintReport } from "./args.ts";
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

function runOxlintVize(root: string, args: readonly string[]) {
  try {
    const stdout = String(
      execFileSync(process.execPath, [cliEntry, ...args], {
        cwd: root,
        encoding: "utf8",
        stdio: "pipe",
      }),
    );
    return { exitCode: 0, stderr: "", stdout };
  } catch (error) {
    const execError = error as {
      status?: number;
      stderr?: string | Buffer;
      stdout?: string | Buffer;
    };
    return {
      exitCode: execError.status ?? 1,
      stderr: String(execError.stderr ?? ""),
      stdout: String(execError.stdout ?? ""),
    };
  }
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
  assert.equal(expectsLintReport(["--format=github"]), true);
  assert.equal(expectsLintReport(["-f", "stylish", "src"]), false);
  assert.equal(expectsLintReport(["-f", "unix"]), false);
  // After `--` everything is a lint target, not an option.
  assert.equal(expectsLintReport(["--", "-f", "json"]), false);
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
  });
});

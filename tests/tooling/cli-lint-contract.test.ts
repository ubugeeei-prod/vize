import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

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

type RunResult = { status: number | null; stdout: string; stderr: string };

function runLint(args: string[], cwd: string): RunResult {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "lint", ...args], {
    cwd,
    encoding: "utf8",
  });
  if (result.error) {
    throw result.error;
  }
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-lint-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// PascalCase file names avoid the vue/component-definition-name-casing rule so
// these fixtures isolate the rule under test.
const MULTI_SPACE = '<template>\n  <div  id="x"></div>\n</template>\n';
const CLEAN = '<template>\n  <div id="x" />\n</template>\n';

test("vize lint --help documents the fix/config/max-warnings surface", () => {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "lint", "--help"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  for (const flag of ["--fix", "--no-config", "--max-warnings", "--format"]) {
    assert.match(
      result.stdout,
      new RegExp(flag.replace(/-/g, "\\-")),
      `help should document ${flag}`,
    );
  }
});

test("vize lint reports a warning but exits 0 by default", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Multi.vue"), MULTI_SPACE, "utf8");
    const result = runLint(["Multi.vue"], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, /no-multi-spaces/);
  });
});

test("vize lint --max-warnings 0 promotes warnings to a failing exit code", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Multi.vue"), MULTI_SPACE, "utf8");
    const result = runLint(["Multi.vue", "--max-warnings", "0"], dir);
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
  });
});

test("vize lint --format json emits a stable per-file message envelope", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Multi.vue"), MULTI_SPACE, "utf8");
    const result = runLint(["Multi.vue", "--format", "json"], dir);

    const parsed = JSON.parse(result.stdout) as Array<{
      file: string;
      errorCount: number;
      warningCount: number;
      messages: Array<{
        ruleId: string;
        severity: number;
        message: string;
        line: number;
        column: number;
        endLine: number;
        endColumn: number;
      }>;
    }>;

    assert.ok(Array.isArray(parsed), result.stdout);
    const entry = parsed.find((item) => item.file.endsWith("Multi.vue"));
    assert.ok(entry, result.stdout);
    assert.equal(entry.errorCount, 0);
    assert.equal(entry.warningCount, 1);

    const message = entry.messages.find((item) => item.ruleId === "vue/no-multi-spaces");
    assert.ok(message, result.stdout);
    assert.equal(message.severity, 1);
    assert.equal(message.line, 2);
    assert.equal(typeof message.column, "number");
    assert.ok(message.endColumn > message.column);
  });
});

test("vize lint exits 0 and reports no problems for a clean file", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Clean.vue"), CLEAN, "utf8");
    const result = runLint(["Clean.vue"], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, /No problems found/);
  });
});

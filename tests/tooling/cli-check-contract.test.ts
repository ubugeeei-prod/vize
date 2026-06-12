import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/**
 * Resolves how to launch the `vize` CLI for contract tests, mirroring the LSP
 * smoke-test launcher: a prebuilt binary wins, otherwise fall back to cargo so
 * fresh checkouts still work.
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
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-check-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

test("vize check --help documents the input contract and key flags", () => {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "check", "--help"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  const help = result.stdout;
  for (const flag of ["--format", "--no-config", "--tsconfig", "--max-warnings", "--quiet"]) {
    assert.match(help, new RegExp(flag.replace(/-/g, "\\-")), `help should document ${flag}`);
  }
  assert.match(help, /PATTERNS/, "help should describe the positional PATTERNS argument");
});

test("vize check exits 1 and reports SFC parse errors before type checking", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Broken.vue"), "<template><div></div>", "utf8");
    const result = runCheck(["Broken.vue"], dir);
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stderr, /SFC parse error/);
  });
});

test("vize check reports parse errors even with config loading disabled", () => {
  // The parse failure must short-circuit before the Corsa type-check stage, so
  // this assertion holds without a type-checker present.
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Bad.vue"), "<template><div>", "utf8");
    const result = runCheck(["Bad.vue", "--no-config"], dir);
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stderr, /SFC parse error/);
  });
});

test("vize check exits 2 when an explicit config file is missing", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Ok.vue"), "<template><p /></template>\n", "utf8");
    const result = runCheck(["Ok.vue", "-c", "./does-not-exist.json"], dir);
    assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stderr, /config file not found/);
  });
});

test("vize check exits 0 when explicit inputs match no supported files", () => {
  withWorkspace((dir) => {
    const result = runCheck(["does-not-exist.vue"], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(result.stdout, "");
    assert.equal(
      result.stderr,
      'No Vue, TypeScript, or JSX files found matching inputs: ["does-not-exist.vue"]\n',
    );
  });
});

test("vize check exits 0 in an empty project with no inputs", () => {
  withWorkspace((dir) => {
    const result = runCheck([], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(result.stdout, "");
    assert.equal(result.stderr, "No Vue, TypeScript, or JSX files found matching inputs: []\n");
  });
});

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
      'No .vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts files found matching inputs: ["does-not-exist.vue"]\n',
    );
  });
});

test("vize check exits 0 in an empty project with no inputs", () => {
  withWorkspace((dir) => {
    const result = runCheck([], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(result.stdout, "");
    assert.equal(
      result.stderr,
      "No .vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts files found matching inputs: []\n",
    );
  });
});

/** Run the real built CLI with no inherited SDK paths, package manager or home. */
function runCheckWithoutRuntime(args: string[], cwd: string): CheckResult {
  assert.equal(VIZE.prefix.length, 0, "isolated runtime contracts require the built source CLI");
  assert(path.isAbsolute(VIZE.command), "isolated runtime contracts require an absolute CLI path");
  const home = path.join(cwd, "isolated-home");
  const bin = path.join(home, "bin");
  fs.mkdirSync(bin, { recursive: true });
  const result = spawnSync(VIZE.command, ["check", "--quiet", ...args], {
    cwd,
    encoding: "utf8",
    timeout: 30_000,
    env: {
      HOME: home,
      USERPROFILE: home,
      XDG_CACHE_HOME: path.join(home, "cache"),
      LOCALAPPDATA: path.join(home, "cache"),
      APPDATA: path.join(home, "config"),
      PATH: bin,
      NO_COLOR: "1",
      ...(process.platform === "win32" ? { SystemRoot: process.env.SystemRoot } : {}),
    },
  });
  if (result.error) throw result.error;
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

for (const extra of [[], ["--no-config"]]) {
  void test(`vize check preserves complete SFC parse failures without an SDK (${extra.length ? "no-config" : "config discovery"})`, () => {
    withWorkspace((dir) => {
      for (const source of [
        "<template><div></div>",
        "<template><div>",
        "\uFEFF<template>\r\n<div>日本語😀\r\n",
      ]) {
        fs.writeFileSync(path.join(dir, "Broken.vue"), source);
        assert.deepEqual(runCheckWithoutRuntime(["Broken.vue", ...extra], dir), {
          status: 1,
          stdout: "",
          stderr:
            "Error: SFC parse error: Malformed <template> block: the closing tag is missing.\n",
        });
      }
    });
  });
}

void test("vize check keeps configuration precedence and complete valid-project missing-runtime help", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Broken.vue"), "<template><div>");
    assert.deepEqual(runCheckWithoutRuntime(["Broken.vue", "--config", "./missing.json"], dir), {
      status: 2,
      stdout: "",
      stderr: "\u001b[31mError:\u001b[0m config file not found: ./missing.json\n",
    });
    fs.writeFileSync(path.join(dir, "Valid.vue"), "<template><p /></template>\n");
    const stderr = [
      "Error: error: native TypeScript runtime not found",
      "",
      "vize check requires a user-installed native TypeScript/tsgo executable.",
      "Keep your existing 'typescript' package. Install a native compiler under a separate alias, or configure typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH.",
      "",
      "The example below selects native TypeScript 7.1.0-beta; choose the native version you want to run.",
      "",
      "Example install commands (choose your package manager):",
      "",
      "  npm install -D typescript-native@npm:typescript@7.1.0-beta  # npm",
      "  pnpm add -D typescript-native@npm:typescript@7.1.0-beta  # pnpm",
      "  yarn add -D typescript-native@npm:typescript@7.1.0-beta  # yarn",
      "  bun add -D typescript-native@npm:typescript@7.1.0-beta  # bun",
      "",
      "",
    ].join("\n");
    for (const extra of [[], ["--no-config"]]) {
      assert.deepEqual(runCheckWithoutRuntime(["Valid.vue", ...extra], dir), {
        status: 1,
        stdout: "",
        stderr,
      });
    }
  });
});

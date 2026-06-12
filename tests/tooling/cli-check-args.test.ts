import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/**
 * Resolves how to launch the `vize` CLI, mirroring `cli-check-contract.test.ts`:
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

// The workspace must live outside the repository tree: `vize check` walks up
// looking for a tsconfig.json, and a repo-local temp dir would inherit the
// workspace tsconfig and start type-checking the repo.
function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-args-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// All of these arg/validation/help paths run BEFORE Corsa discovery: each
// passing-a-real-file case routes through `--corsa-path /nonexistent` to prove
// the binary never reaches the type checker, so the assertions are stable in a
// clean checkout with no Corsa/tsgo on disk.

test("vize check rejects --servers 0 and --servers 64 with exit 2 before corsa discovery", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "a.vue"), "<template><p /></template>\n", "utf8");

    const zero = runCheck(["a.vue", "--servers", "0", "--corsa-path", "/nonexistent"], dir);
    assert.equal(zero.status, 2, `${zero.stdout}\n${zero.stderr}`);
    assert.match(zero.stderr, /servers must be at least 1/);

    const tooMany = runCheck(["a.vue", "--servers", "64", "--corsa-path", "/nonexistent"], dir);
    assert.equal(tooMany.status, 2, `${tooMany.stdout}\n${tooMany.stderr}`);
    assert.match(tooMany.stderr, /servers=64 exceeds the supported maximum/);
  });
});

test("vize check exits 2 and echoes the missing explicit -c config path", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "a.vue"), "<template><p /></template>\n", "utf8");
    const result = runCheck(
      ["a.vue", "-c", "./missing.config.ts", "--corsa-path", "/nonexistent"],
      dir,
    );
    assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stderr, /config file not found/);
    assert.ok(result.stderr.includes("./missing.config.ts"), result.stderr);
  });
});

test("vize check --no-config bypasses validation of an invalid -c path", () => {
  withWorkspace((dir) => {
    // No matching files keeps the run corsa-free; --no-config must skip the
    // explicit-config validation that would otherwise exit 2.
    const result = runCheck(["zzz.vue", "--no-config", "-c", "./missing.config.ts"], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(result.stdout, "");
    assert.equal(
      result.stderr,
      'No Vue, TypeScript, or JSX files found matching inputs: ["zzz.vue"]\n',
    );
  });
});

test("vize check skips and exits 0 when typeChecker.enabled is false (JSON config)", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "a.vue"), "<template><p /></template>\n", "utf8");
    // JSON config (not .ts) so it resolves without a JS runtime in the temp dir.
    fs.writeFileSync(
      path.join(dir, "vize.config.json"),
      JSON.stringify({ typeChecker: { enabled: false } }),
      "utf8",
    );
    const result = runCheck(["a.vue", "--corsa-path", "/nonexistent"], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.match(
      `${result.stdout}${result.stderr}`,
      /Skipping check because typeChecker\.enabled is false/,
    );
  });
});

test("vize check surfaces clap errors with exit 2 while --help exits 0", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "a.vue"), "<template><p /></template>\n", "utf8");

    const bogus = runCheck(["a.vue", "--bogus"], dir);
    assert.equal(bogus.status, 2, `${bogus.stdout}\n${bogus.stderr}`);
    assert.match(bogus.stderr, /unexpected argument '--bogus'/);

    const noValue = runCheck(["a.vue", "--format"], dir);
    assert.equal(noValue.status, 2, `${noValue.stdout}\n${noValue.stderr}`);

    const help = runCheck(["--help"], dir);
    assert.equal(help.status, 0, `${help.stdout}\n${help.stderr}`);
    assert.match(help.stdout, /Usage: vize check/);
  });
});

test("vize check --help documents the declaration/tsconfig/format/config option surface", () => {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "check", "--help"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  const help = result.stdout;
  for (const fragment of [
    "--tsconfig <TSCONFIG>",
    "--format <FORMAT>",
    "[default: text]",
    "--max-warnings <MAX_WARNINGS>",
    "--no-config",
    "-c, --config <CONFIG>",
    "--declaration",
    "--declaration-dir <DECLARATION_DIR>",
    "--corsa-path <CORSA_PATH>",
    "--quiet",
  ]) {
    assert.ok(help.includes(fragment), `check --help should document ${fragment}`);
  }
});

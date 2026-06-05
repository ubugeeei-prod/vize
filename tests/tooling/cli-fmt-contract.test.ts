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

function runFmt(args: string[], cwd: string): RunResult {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "fmt", ...args], {
    cwd,
    encoding: "utf8",
  });
  if (result.error) {
    throw result.error;
  }
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-fmt-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

const UNFORMATTED = "<template><div   /></template>\n";

test("vize fmt --help documents the check/write contract and default pattern", () => {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "fmt", "--help"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /--check/);
  assert.match(result.stdout, /--write/);
  assert.match(result.stdout, /\*\*\/\*\.vue/, "should document the default ./**/*.vue glob");
});

test("vize fmt --check exits 1 and names files that need reformatting", () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Unformatted.vue"), UNFORMATTED, "utf8");
    const result = runFmt(["--check", "Unformatted.vue"], dir);
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, /Unformatted\.vue/);
    assert.match(`${result.stdout}${result.stderr}`, /reformat/i);
  });
});

test("vize fmt --write rewrites the file so a follow-up --check passes", () => {
  withWorkspace((dir) => {
    const file = path.join(dir, "Rewrite.vue");
    fs.writeFileSync(file, UNFORMATTED, "utf8");

    const write = runFmt(["--write", "Rewrite.vue"], dir);
    assert.equal(write.status, 0, `${write.stdout}\n${write.stderr}`);

    // The file content must actually change, and the formatter must be
    // idempotent: re-checking the written file reports no work.
    const rewritten = fs.readFileSync(file, "utf8");
    assert.notEqual(rewritten, UNFORMATTED);

    const recheck = runFmt(["--check", "Rewrite.vue"], dir);
    assert.equal(recheck.status, 0, `${recheck.stdout}\n${recheck.stderr}`);
  });
});

test("vize fmt --check exits 0 on an already-formatted file", () => {
  withWorkspace((dir) => {
    const file = path.join(dir, "Already.vue");
    fs.writeFileSync(file, UNFORMATTED, "utf8");
    runFmt(["--write", "Already.vue"], dir);

    const recheck = runFmt(["--check", "Already.vue"], dir);
    assert.equal(recheck.status, 0, `${recheck.stdout}\n${recheck.stderr}`);
  });
});

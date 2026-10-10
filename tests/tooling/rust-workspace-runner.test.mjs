import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

const runner = new URL(
  "../../tools/support/compat/github/run-rust-workspace-tests.mjs",
  import.meta.url,
);

function fixture(mode, failure = "", version = "0.9.146") {
  const cwd = mkdtempSync(join(tmpdir(), "vize-workspace-runner-"));
  const calls = join(cwd, "calls.jsonl");
  const receipt = join(cwd, "receipt.json");
  const cargo = join(cwd, "cargo");
  writeFileSync(
    cargo,
    `#!${process.execPath}\nconst fs = require('node:fs'); const args = process.argv.slice(2); if (args.join(' ') === 'nextest --version') { console.log('cargo-nextest ${version}'); } else { fs.appendFileSync(process.env.CALLS, JSON.stringify(args)+'\\n'); if (args.includes(process.env.FAIL)) process.exit(37); }\n`,
    { mode: 0o755 },
  );
  const result = spawnSync(
    process.execPath,
    [runner.pathname, "--runner", mode, "--output", receipt],
    {
      cwd,
      encoding: "utf8",
      env: { ...process.env, PATH: `${cwd}:${process.env.PATH}`, CALLS: calls, FAIL: failure },
    },
  );
  const observed = {
    result,
    calls: (() => {
      try {
        return readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse);
      } catch {
        return [];
      }
    })(),
    receipt: (() => {
      try {
        return JSON.parse(readFileSync(receipt, "utf8"));
      } catch {
        return null;
      }
    })(),
  };
  rmSync(cwd, { recursive: true, force: true });
  return observed;
}

void test("full nextest keeps workspace selection, pinned version, and explicit doctests", () => {
  const f = fixture("nextest");
  assert.equal(f.result.status, 0, f.result.stderr);
  assert.deepEqual(f.calls, [
    ["nextest", "run", "--locked", "--workspace", "--profile", "full"],
    ["test", "--locked", "--workspace", "--doc"],
  ]);
  assert.equal(f.receipt.runner, "nextest");
  assert.equal(f.receipt.status, "passed");
  assert.equal(f.receipt.phases.length, 2);
});

void test("serial reference invokes the original complete cargo workspace recipe", () => {
  const f = fixture("cargo");
  assert.equal(f.result.status, 0, f.result.stderr);
  assert.deepEqual(f.calls, [["test", "--locked", "--workspace"]]);
});

void test("failure in either ordinary tests or doctests fails the complete receipt", () => {
  for (const phase of ["nextest", "--doc"]) {
    const f = fixture("nextest", phase);
    assert.equal(f.result.status, 37, f.result.stderr);
    assert.equal(f.receipt.status, "failed");
    assert.equal(f.calls.length, 2, "doctests remain attempted after an ordinary-test failure");
    assert.equal(f.receipt.phases.find((entry) => entry.exitCode === 37).status, "failed");
  }
});

void test("unknown runner or foreign nextest fails before executing test commands", () => {
  for (const [mode, version] of [
    ["unknown", "0.9.146"],
    ["nextest", "0.9.145"],
  ]) {
    const f = fixture(mode, "", version);
    assert.notEqual(f.result.status, 0);
    assert.deepEqual(f.calls, []);
  }
});

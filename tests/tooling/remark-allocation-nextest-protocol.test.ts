import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const CASE = "remarks_preserve_zero_cost_and_the_positive_control";

test("remark allocation harness separates nextest discovery, selection and callback failure", () => {
  const source = readFileSync(
    fileURLToPath(new URL("../../davinci/vize_l0/tests/remark_zero_cost.rs", import.meta.url)),
    "utf8",
  );
  const start = source.indexOf("const CASE: &str =");
  const end = source.indexOf("fn remarks_preserve_zero_cost_and_the_positive_control()", start);
  assert.ok(start >= 0 && end > start, "compile the actual single-case CLI");
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-allocation-nextest-protocol-"));
  try {
    const filename = path.join(root, "harness.rs");
    const binary = path.join(root, "harness");
    // Compile the real CASE and main unchanged. Only the measurement callback
    // is instrumented; the real allocation windows are exercised by Rust CI.
    writeFileSync(
      filename,
      source.slice(start, end) +
        `fn remarks_preserve_zero_cost_and_the_positive_control() {
    println!("measurement callback entered");
    assert_ne!(
        std::env::var("VIZE_PROTOCOL_CALLBACK_FAIL").as_deref(),
        Ok("1"),
        "injected measurement callback assertion"
    );
}
`,
    );
    execFileSync("rustc", ["--edition=2024", filename, "-o", binary], { stdio: "pipe" });
    const run = (args: string[], fail = false) => {
      const result = spawnSync(binary, args, {
        encoding: "utf8",
        env: { ...process.env, VIZE_PROTOCOL_CALLBACK_FAIL: fail ? "1" : "0" },
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      return result;
    };
    for (const args of [
      ["--list", "--format", "terse"],
      [CASE, "--list", "--format", "terse", "--exact"],
    ]) {
      const result = run(args, true);
      assert.equal(result.status, 0, "discovery must not execute a failing callback");
      assert.equal(result.stdout, `${CASE}: test\n`);
      assert.equal(result.stderr, "");
    }
    for (const args of [
      ["--list", "--format", "terse", "--ignored"],
      ["--ignored"],
      ["missing", "--nocapture", "--exact"],
      ["--list", "--format", "terse", "missing"],
      ["remarks_preserve", "--nocapture", "--exact"],
    ]) {
      const result = run(args, true);
      assert.equal(result.status, 0, "excluded selection must not enter the callback");
      assert.equal(result.stdout, "");
      assert.equal(result.stderr, "");
    }
    for (const args of [[], [CASE, "--nocapture", "--exact"], ["remarks_preserve"]]) {
      const result = run(args);
      assert.equal(result.status, 0);
      assert.equal(result.stdout, "measurement callback entered\n");
      assert.equal(result.stderr, "");
      const failed = run(args, true);
      assert.notEqual(failed.status, 0, "callback assertion failures must fail the selected case");
      assert.equal(failed.stdout, "measurement callback entered\n");
      assert.match(failed.stderr, /injected measurement callback assertion/u);
    }
    for (const args of [["--unsupported"], ["--format"], ["--format", "pretty"]]) {
      const result = run(args, true);
      assert.notEqual(result.status, 0);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, /unsupported harness argument/u);
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

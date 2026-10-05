import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const CASE = "native_lane_stays_within_its_allocation_ceilings";

test("Vapor allocation harness separates nextest discovery, selection and callback failure", () => {
  const source = readFileSync(
    fileURLToPath(
      new URL(
        "../../crates/vize_atelier_vapor/tests/davinci_vapor_native_budget.rs",
        import.meta.url,
      ),
    ),
    "utf8",
  );
  const start = source.indexOf("const CASE: &str =");
  const end = source.indexOf("fn native_lane_stays_within_its_allocation_ceilings()", start);
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
        `fn native_lane_stays_within_its_allocation_ceilings() {
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
      ["native_lane_stays", "--nocapture", "--exact"],
    ]) {
      const result = run(args, true);
      assert.equal(result.status, 0, "excluded selection must not enter the callback");
      assert.equal(result.stdout, "");
      assert.equal(result.stderr, "");
    }
    for (const args of [[], [CASE, "--nocapture", "--exact"], ["native_lane_stays"]]) {
      const result = run(args);
      assert.equal(result.status, 0);
      assert.equal(result.stdout, "measurement callback entered\n");
      assert.equal(result.stderr, "");
      const failed = run(args, true);
      assert.notEqual(failed.status, 0, "callback assertion failures must fail the selected case");
      assert.equal(failed.stdout, "measurement callback entered\n");
      assert.match(failed.stderr, /injected measurement callback assertion/u);
    }
    for (const args of [
      ["--unsupported"],
      ["--format"],
      ["--format", "pretty"],
      ["--test-threads=4"],
      ["--test-threads", "4"],
    ]) {
      const result = run(args, true);
      assert.notEqual(result.status, 0);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, /unsupported harness argument/u);
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("historical diagnostic rejects an incompatible overlay before archive or process work", () => {
  execFileSync(
    "python3",
    [
      fileURLToPath(
        new URL(
          "../../tools/support/compat/github/allocation_ownership/test_verify.py",
          import.meta.url,
        ),
      ),
    ],
    { stdio: "pipe" },
  );
});

test("Vapor budget remains one mandatory, output-retained nextest case", () => {
  const manifest = readFileSync(
    fileURLToPath(new URL("../../crates/vize_atelier_vapor/Cargo.toml", import.meta.url)),
    "utf8",
  );
  const config = readFileSync(
    fileURLToPath(new URL("../../.config/nextest.toml", import.meta.url)),
    "utf8",
  );
  const target = manifest
    .split("[[test]]")
    .find((entry) => entry.split(/\n\s*\n/u)[0]?.includes('name = "davinci_vapor_native_budget"'));
  assert.ok(target, "the unchanged target is explicitly registered");
  const registration = target.split(/\n\s*\n/u)[0];
  assert.match(registration ?? "", /harness = false/u);
  assert.doesNotMatch(registration ?? "", /required-features|test = false/u);
  const expected = `package(=vize_atelier_vapor) & binary(=davinci_vapor_native_budget) & test(=${CASE})`;
  const override = config
    .split("[[profile.default.overrides]]")
    .find((entry) => entry.split(/\n\s*\n/u)[0]?.includes(`filter = '${expected}'`));
  assert.ok(override, "only the complete original case retains successful output");
  const output = override.split(/\n\s*\n/u)[0];
  assert.match(output ?? "", /junit\.store-success-output = true/u);
  assert.doesNotMatch(output ?? "", /retries|test-group|threads-required|run-extra-args/u);
});

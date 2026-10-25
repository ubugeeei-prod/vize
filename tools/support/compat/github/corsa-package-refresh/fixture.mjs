import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { validateSourceProvenance } from "./source.mjs";

assert.equal(process.env.GITHUB_ACTIONS, "true");
const capture = process.env.VIZE_PACKAGE_REFRESH_CAPTURE;
assert(capture);
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const lifecycle = JSON.parse(fs.readFileSync(path.join(capture, "receipt.json"), "utf8"));
const source = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
assert.equal(source, lifecycle.observer);
const provenance = validateSourceProvenance(lifecycle.provenance, source);
validateSourceProvenance(
  JSON.parse(fs.readFileSync(path.join(capture, "source.json"), "utf8")),
  source,
);
assert.equal(hash(lifecycle.executable), lifecycle.sha256);
const file = "crates/vize_canon/tests/fix_history_diagnostics.rs";
const original = fs.readFileSync(file, "utf8");
const needle = "        let actual: Vec<Diagnostic> = result";
assert.equal(original.split(needle).length, 2);
try {
  // Read-only observation of the complete production result, before its unchanged
  // exact diagnostic comparison. This is focused evidence, not a full-worker receipt.
  fs.writeFileSync(
    file,
    original.replace(needle, '        eprintln!("OBSERVED_BATCH {result:#?}");\n' + needle),
  );
  const build = spawnSync(
    "cargo",
    [
      "test",
      "--test",
      "fix_history_diagnostics",
      "--no-run",
      "--message-format=json",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize_canon",
    ],
    { encoding: "utf8", timeout: 900_000, maxBuffer: 32_000_000 },
  );
  fs.writeFileSync(path.join(capture, "fixture-build.stdout"), build.stdout || "");
  fs.writeFileSync(path.join(capture, "fixture-build.stderr"), build.stderr || "");
  assert.equal(build.error, undefined);
  assert.equal(build.status, 0, build.stderr);
  const binaries = build.stdout
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        row.target.name === "fix_history_diagnostics" &&
        row.profile.test &&
        row.executable,
    );
  assert.equal(binaries.length, 1);
  const binary = fs.realpathSync(binaries[0].executable);
  const selector = "missing_exported_package_target_preserves_ts2307";
  const env = { ...process.env, CORSA_PATH: lifecycle.executable };
  delete env.VIZE_TEST_DISABLE_TSGO;
  const result = spawnSync(binary, ["--exact", selector, "--nocapture"], {
    env,
    encoding: "utf8",
    timeout: 180_000,
    maxBuffer: 32_000_000,
  });
  fs.writeFileSync(path.join(capture, "fixture.stdout"), result.stdout || "");
  fs.writeFileSync(path.join(capture, "fixture.stderr"), result.stderr || "");
  const fixtureRoot = "tests/_fixtures/differential/typechecker/package-target-deletion-refresh";
  fs.cpSync(fixtureRoot, path.join(capture, "pinned-cold-input"), { recursive: true });
  fs.writeFileSync(
    path.join(capture, "fixture-receipt.json"),
    JSON.stringify(
      {
        kind: "focused-batch-observation",
        source,
        provenance,
        selector,
        binary,
        binarySha256: hash(binary),
        sdk: {
          path: lifecycle.executable,
          sha256: hash(lifecycle.executable),
          version: lifecycle.version,
        },
        exit: result.status,
        signal: result.signal,
        error: result.error?.message,
        inputs: fs
          .readdirSync(fixtureRoot)
          .sort()
          .map((name) => ({ name, sha256: hash(path.join(fixtureRoot, name)) })),
      },
      null,
      2,
    ),
  );
  assert.equal(result.error, undefined);
  assert.match(result.stdout, /running 1 test/);
  assert.match(result.stderr, /OBSERVED_BATCH/);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /test result: ok/);
} finally {
  fs.writeFileSync(file, original);
}

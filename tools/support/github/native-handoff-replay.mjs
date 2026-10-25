import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const directory = resolve(process.argv[2]);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const file = (path) => ({ path, sha256: sha256(readFileSync(path)) });
const source = readFileSync(join(directory, "source.sha"), "utf8").trim();
assert.match(source, /^[0-9a-f]{40}$/);
assert.equal(source, process.env.SOURCE_SHA);

const packageRoot = realpathSync("node_modules/@typescript/typescript-linux-x64");
const manifestPath = join(packageRoot, "package.json");
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
assert.equal(manifest.name, "@typescript/typescript-linux-x64");
assert.equal(manifest.version, "7.0.2");
const native = realpathSync(join(packageRoot, "lib/tsc"));
assert.equal(
  sha256(readFileSync(native)),
  "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
);
const version = spawnSync(native, ["--version"], { encoding: "utf8", timeout: 10000 });
assert.equal(version.status, 0);
assert.equal(version.stdout, "Version 7.0.2\n");
assert.equal(version.stderr, "");

const artifacts = readFileSync(join(directory, "build.stdout"), "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line))
  .filter(
    (event) =>
      event.reason === "compiler-artifact" &&
      event.target.name === "native_syntax_semantics" &&
      event.executable,
  );
assert.equal(artifacts.length, 1, "the exact original test binary must be built once");
const executable = realpathSync(artifacts[0].executable);
const name = "handoff_notice::required_semantic_completion_does_not_report_runtime_degradation";
const argv = ["--exact", name, "--nocapture", "--test-threads=1", "--color", "never"];
const environment = {
  ...process.env,
  CORSA_PATH: native,
  VIZE_TEST_REQUIRE_TSGO: "1",
  VIZE_TEST_REQUIRE_PIN_INPUT_FIXTURE: "1",
};
delete environment.VIZE_TEST_DISABLE_TSGO;
delete environment.VIZE_SILENCE_CORSA_FALLBACK;
delete environment.VIZE_TEST_HANDOFF_NOTICE_CAPTURE;
const result = spawnSync(executable, argv, {
  env: environment,
  timeout: 120000,
  maxBuffer: 16 * 1024 * 1024,
});
const stdout = result.stdout ?? Buffer.alloc(0);
const stderr = result.stderr ?? Buffer.alloc(0);
writeFileSync(join(directory, "test.stdout"), stdout);
writeFileSync(join(directory, "test.stderr"), stderr);
const packet = {
  source,
  driver: readFileSync(join(directory, "driver.sha"), "utf8").trim(),
  runtime: { ...file(native), manifest, manifestFile: file(manifestPath), version: version.stdout },
  originalTest: { ...file(executable), argv },
  compiler: spawnSync("rustc", ["--version", "--verbose"], { encoding: "utf8" }).stdout,
  sources: [
    "Cargo.lock",
    "pnpm-lock.yaml",
    "crates/vize_canon/tests/native_syntax_semantics/main.rs",
    "crates/vize_canon/tests/native_syntax_semantics/handoff_notice.rs",
    "crates/vize_canon/src/lsp_client/diagnostics_lsp.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown.rs",
  ].map(file),
  observation: {
    exitCode: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
    wholeStdout: stdout.toString("utf8"),
    wholeStderr: stderr.toString("utf8"),
  },
};
writeFileSync(join(directory, "observation.json"), JSON.stringify(packet, null, 2) + "\n");
process.stdout.write(stdout);
process.stderr.write(stderr);
assert.equal(result.error, undefined);
assert.equal(result.signal, null);
assert.equal(result.status, 0, "the unchanged strict original handoff law must pass");
assert.match(stdout.toString("utf8"), new RegExp(`test ${name} \\.\\.\\. ok`));
assert.match(stdout.toString("utf8"), /^test result: ok\. 1 passed; 0 failed; 0 ignored; /m);

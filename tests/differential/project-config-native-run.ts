import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.ts";
import { sha256 } from "./sha256.ts";

export const PREPARE_ARGV = [
  "cargo",
  "test",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize",
  "--test",
  "project_config_cli",
  "--no-run",
  "--message-format=json",
];
export const BUILD_ARGV = ["cargo", "build", "--locked", "--profile", "ci", "-p", "vize"];

// Cargo test can relink the regular CLI with dev-dependency features. Prepare
// its exact named harness first, seal the canonical production build second,
// then execute that harness without another Cargo invocation.
export function runPreparedProjectContracts(root: string): void {
  const destination = path.join(root, "target/project-config-native");
  const identity = expectedBuildIdentity(root);
  assert.equal(identity.sourceRevision, process.env.SOURCE_SHA);
  assert.equal(process.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(process.env.VIZE_TEST_DISABLE_TSGO, undefined);
  validateBuildReceipt(
    JSON.parse(fs.readFileSync(path.join(destination, "build.json"), "utf8")),
    identity,
  );
  const preparation = fs.readFileSync(path.join(destination, "prepare-tests.jsonl"));
  const artifacts = preparation
    .toString()
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const selected = artifacts.filter(
    (artifact) =>
      artifact.reason === "compiler-artifact" &&
      artifact.target.name === "project_config_cli" &&
      artifact.target.kind.length === 1 &&
      artifact.target.kind[0] === "test" &&
      artifact.profile.test === true &&
      typeof artifact.executable === "string" &&
      artifact.manifest_path === path.join(root, "crates/vize/Cargo.toml") &&
      artifact.target.src_path === path.join(root, "crates/vize/tests/project_config_cli.rs"),
  );
  assert.equal(selected.length, 1, "exact Cargo-authored project_config_cli harness is required");
  const [packagePath, packageVersion] = selected[0].package_id.split("#");
  assert.equal(packagePath, `path+${pathToFileURL(path.join(root, "crates/vize")).href}`);
  const version = identity.cliVersion.slice("vize ".length);
  assert.ok(packageVersion === version || packageVersion === `vize@${version}`);
  const executable = fs.realpathSync(selected[0].executable);
  const artifactRoot = fs.realpathSync(path.join(root, "target/ci/deps"));
  assert.equal(path.dirname(executable), artifactRoot);
  const harnessSha256 = sha256(fs.readFileSync(executable));
  const arguments_ = ["--nocapture"];
  const result = spawnSync(executable, arguments_, {
    cwd: root,
    env: process.env,
    maxBuffer: 64 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  const harnessAfterSha256 = sha256(fs.readFileSync(executable));
  const productionAfterIdentity = expectedBuildIdentity(root);
  fs.writeFileSync(path.join(destination, "tests.log"), stdout);
  fs.writeFileSync(path.join(destination, "harness-stderr.bin"), stderr);
  fs.writeFileSync(
    path.join(destination, "harness.json"),
    `${JSON.stringify(
      {
        ...identity,
        prepareArgv: PREPARE_ARGV,
        executable,
        harnessSha256,
        harnessAfterSha256,
        productionAfterIdentity,
        cargoArtifact: selected[0],
        arguments: arguments_,
        exitCode: result.status,
        signal: result.signal,
        error: result.error?.message,
        preparationSha256: sha256(preparation),
        stdoutSha256: sha256(stdout),
        stderrSha256: sha256(stderr),
        stderrBytes: stderr.length,
      },
      null,
      2,
    )}\n`,
  );
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  assert.equal(harnessAfterSha256, harnessSha256);
  assert.deepEqual(productionAfterIdentity, identity, "sealed production CLI remains unchanged");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 2);
  runPreparedProjectContracts(process.cwd());
}

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { compileProductObserver, observerBuildArgs } from "./observer-build.ts";

export const OBSERVER_SOURCE = "crates/vize_glyph/examples/formatter_observe.rs";
const observerSpec = {
  product: "formatter",
  packageName: "vize_glyph",
  exampleName: "formatter_observe",
  sourcePath: OBSERVER_SOURCE,
  probes: [["--defaults"], ["--defaults", "--legacy-single-pass"]],
};
function buildArgs(profile, offline) {
  return observerBuildArgs(observerSpec, profile, offline);
}

function git(root, args) {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function observerSourceIdentity(repoRoot) {
  const untrackedSources = git(repoRoot, [
    "ls-files",
    "--others",
    "--exclude-standard",
    "-z",
    "--",
    "crates",
  ])
    .split("\0")
    .filter((file) => file && !file.includes("/tests/") && /(?:\.rs|Cargo\.toml)$/.test(file));
  assert.equal(
    untrackedSources.length,
    0,
    "untracked Rust product sources cannot enter a source-built observation",
  );
  const productionChanges = git(repoRoot, [
    "diff",
    "--name-only",
    "-z",
    "HEAD",
    "--",
    "Cargo.lock",
    "Cargo.toml",
    "crates",
  ])
    .split("\0")
    .filter((file) => file && !file.includes("/tests/") && file !== OBSERVER_SOURCE);
  assert.equal(
    productionChanges.length,
    0,
    "source-built observation requires unchanged product sources",
  );
  return {
    sourceRevision: git(repoRoot, ["rev-parse", "HEAD"]),
    formatterSourceTree: git(repoRoot, ["rev-parse", "HEAD:crates/vize_glyph/src"]),
    cargoLockSha256: sha256(fs.readFileSync(path.join(repoRoot, "Cargo.lock"))),
    observerSourceSha256: sha256(fs.readFileSync(path.join(repoRoot, OBSERVER_SOURCE))),
  };
}

function toolchainVersion(program, root) {
  const result = spawnSync(program, ["--version"], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function validateObserverReceipt(receipt, repoRoot, binaryPath) {
  assert.equal(receipt.schema, "vize.formatter-api-build");
  assert.equal(receipt.version, 1);
  assert.deepEqual(receipt.source, observerSourceIdentity(repoRoot));
  assert.deepEqual(receipt.command, ["cargo", ...buildArgs(receipt.profileName, receipt.offline)]);
  assert.equal(receipt.artifact.target.name, "formatter_observe");
  assert.deepEqual(receipt.artifact.target.kind, ["example"]);
  assert.equal(fs.realpathSync(receipt.artifact.executable), fs.realpathSync(binaryPath));
  assert.equal(receipt.artifact.sha256, sha256(fs.readFileSync(binaryPath)));
  assert.equal(receipt.artifact.profile.test, false);
  assert.deepEqual(receipt.artifact.features, []);
  assert.equal(receipt.exitStatus, 0);
  assert.deepEqual(receipt.toolchain, {
    rustc: toolchainVersion("rustc", repoRoot),
    cargo: toolchainVersion("cargo", repoRoot),
  });
  assert.equal(receipt.options.length, 2);
  for (const [index, observation] of receipt.options.entries()) {
    assert.deepEqual(observation.argv, ["--defaults", ...(index ? ["--legacy-single-pass"] : [])]);
    assert.equal(observation.exitStatus, 0);
    const bytes = Buffer.from(observation.stdoutBase64, "base64");
    assert.equal(sha256(bytes), observation.sha256);
    assert(bytes.toString().includes(`skip_script_stabilization: ${Boolean(index)}`));
    const actual = spawnSync(binaryPath, observation.argv, { timeout: 30_000 });
    assert.equal(actual.status, 0);
    assert.equal(actual.signal, null);
    assert(
      actual.stdout.equals(bytes) && actual.stderr.length === 0,
      "actual options probe changed",
    );
  }
  return receipt;
}

// Build and capture are real operations. A pre-existing binary never substitutes
// for a successful Cargo artifact emitted by this source checkout.
export function buildFormatterObserver({
  repoRoot,
  targetDir,
  evidenceDir,
  profile = "dev",
  offline = false,
}) {
  assert(path.isAbsolute(targetDir) && path.isAbsolute(evidenceDir));
  const source = observerSourceIdentity(repoRoot);
  fs.mkdirSync(evidenceDir, { recursive: true });
  const argv = buildArgs(profile, offline);
  const built = compileProductObserver({
    spec: observerSpec,
    repoRoot,
    targetDir,
    evidenceDir,
    profile,
    offline,
  });
  const frozen = built.binaryPath;
  const options = [false, true].map((singlePass) => {
    const argv = ["--defaults", ...(singlePass ? ["--legacy-single-pass"] : [])];
    const result = spawnSync(frozen, argv, { timeout: 30_000 });
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    assert.equal(result.stderr.length, 0);
    return {
      argv,
      exitStatus: result.status,
      stdoutBase64: result.stdout.toString("base64"),
      sha256: sha256(result.stdout),
    };
  });
  const receipt = {
    schema: "vize.formatter-api-build",
    version: 1,
    source,
    command: ["cargo", ...argv],
    profileName: profile,
    offline,
    toolchain: {
      rustc: toolchainVersion("rustc", repoRoot),
      cargo: toolchainVersion("cargo", repoRoot),
    },
    options,
    exitStatus: built.exitStatus,
    artifact: {
      target: built.artifact.target,
      profile: built.artifact.profile,
      features: built.artifact.features,
      executable: frozen,
      cargoExecutable: built.artifact.cargoExecutable,
      sha256: sha256(fs.readFileSync(frozen)),
    },
    logs: built.logs,
  };
  assert.deepEqual(observerSourceIdentity(repoRoot), source, "source changed during build");
  validateObserverReceipt(receipt, repoRoot, frozen);
  fs.writeFileSync(
    path.join(evidenceDir, "build-receipt.json"),
    `${JSON.stringify(receipt, null, 2)}\n`,
  );
  return { binaryPath: frozen, receipt };
}

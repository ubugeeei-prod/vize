import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import {
  SHIPPING_BUILD_RECIPE,
  BUILD_RECIPE,
  writeBuildReceipt,
} from "../../differential/build-receipt.ts";
import { driverRoot, git, sha256, sourceIdentity } from "./warm-type-backed-source.ts";
import {
  assertChangedDependenciesBuilt,
  changedSourceDependencies,
} from "./warm-type-backed-dependency-custody.ts";

const side = process.argv[2];
assert.ok(side === "before" || side === "after");
assert.equal(process.platform, "linux");
assert.ok(process.env.RUNNER_TEMP && process.env.GITHUB_WORKSPACE);
assert.equal(fs.realpathSync(process.env.GITHUB_WORKSPACE), driverRoot);
const output = path.join(process.env.RUNNER_TEMP, "warm-pair");
const before = path.join(process.env.RUNNER_TEMP, "warm-before");
const binding = JSON.parse(fs.readFileSync(path.join(output, "workflow-source.json"), "utf8"));
assert.deepEqual(sourceIdentity(driverRoot), binding.driverSource);
const root = side === "before" ? before : (binding.afterRoot ?? driverRoot);
assert.equal(git(root, ["rev-parse", "HEAD"]), side === "before" ? binding.baseline : binding.head);
assert.equal(sourceIdentity(root).dirty, "");
const sourceBefore = sourceIdentity(root);
const target = path.join(driverRoot, "target");
const profile = process.env.WARM_REQUEST_BUILD_PROFILE ?? "ci";
assert.ok(profile === "ci" || profile === "release");
const binary = path.join(target, profile, "vize");
const environment = { ...process.env, CARGO_TARGET_DIR: target };

function cargo(name: string, args: string[], command = "cargo") {
  const result = spawnSync(command, args, {
    cwd: root,
    env: environment,
    maxBuffer: 64 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(output, `${side}-${name}.stdout`), result.stdout ?? "");
  fs.writeFileSync(path.join(output, `${side}-${name}.stderr`), result.stderr ?? "");
  fs.writeFileSync(
    path.join(output, `${side}-${name}.process.json`),
    `${JSON.stringify({ command, args, cwd: root, target, status: result.status, signal: result.signal, error: result.error && String(result.error) }, null, 2)}\n`,
  );
  process.stderr.write(result.stderr ?? "");
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, `${side} ${name} must succeed`);
  const stdout = result.stdout.toString("utf8");
  assert.ok(Buffer.from(stdout).equals(result.stdout), "Cargo JSON must be lossless UTF-8");
  return stdout;
}

// Finite cuts clean the complete selected profile. The owned-source and harness-only
// routes rebuild Canon, its Maestro/CLI consumers and changed qualified dependencies.
const changedDependencies = changedSourceDependencies(binding.production);
const clean =
  binding.authority === "root-frozen-release-cut"
    ? ["clean", "--profile", profile, "--locked"]
    : [
        "clean",
        "--profile",
        profile,
        "--locked",
        "-p",
        "vize_canon",
        "-p",
        "vize_maestro",
        "-p",
        "vize",
        ...changedDependencies.flatMap((name) => ["-p", name]),
      ];
const toolchain = {
  cargo: cargo("toolchain-cargo", ["--version"]),
  rustc: cargo("toolchain-rustc", ["--version", "--verbose"], "rustc"),
};
cargo("clean", clean);
assert.ok(!fs.existsSync(binary), "package clean must remove the previous CLI");
const build = [
  "build",
  ...(profile === "release" ? ["--release"] : ["--profile", "ci"]),
  "-p",
  "vize",
  "--locked",
  "--message-format=json",
];
const messages = cargo("cargo", build)
  .split("\n")
  .filter((line) => line.startsWith("{"))
  .map((line) => JSON.parse(line));
const finished = messages.filter((entry) => entry.reason === "build-finished");
assert.equal(finished.length, 1);
assert.equal(finished[0].success, true);
const localArtifacts = messages
  .filter((entry) => entry.reason === "compiler-artifact" && entry.profile.test === false)
  .filter((entry) => entry.manifest_path.startsWith(`${root}${path.sep}`))
  .map((entry) => {
    assert.ok(fs.realpathSync(entry.manifest_path).startsWith(`${root}${path.sep}`));
    assert.ok(fs.realpathSync(entry.target.src_path).startsWith(`${root}${path.sep}`));
    if (binding.authority === "root-frozen-release-cut")
      assert.equal(entry.fresh, false, "every linked local artifact must actually compile");
    return {
      ...entry,
      manifestSha256: sha256(fs.readFileSync(entry.manifest_path)),
      sourceSha256: sha256(fs.readFileSync(entry.target.src_path)),
    };
  });
assert.ok(localArtifacts.length >= 4);
assertChangedDependenciesBuilt(changedDependencies, localArtifacts);
const required = [
  { name: "vize_canon", kind: "lib", file: "lib.rs", features: ["native"] },
  { name: "vize_maestro", kind: "lib", file: "lib.rs", features: ["default", "glyph", "native"] },
  { name: "vize", kind: "lib", file: "lib.rs", features: ["default", "glyph", "maestro"] },
  { name: "vize", kind: "bin", file: "main.rs", features: ["default", "glyph", "maestro"] },
];
const artifacts = required.map((expected) => {
  const selected = messages.filter(
    (entry) =>
      entry.reason === "compiler-artifact" &&
      entry.target.name === expected.name &&
      entry.target.kind.length === 1 &&
      entry.target.kind[0] === expected.kind &&
      entry.profile.test === false,
  );
  assert.equal(selected.length, 1, `one ${expected.name} ${expected.kind} artifact is required`);
  const artifact = selected[0];
  const manifest = path.join(root, "crates", expected.name, "Cargo.toml");
  const source = path.join(root, "crates", expected.name, "src", expected.file);
  assert.equal(artifact.manifest_path, manifest);
  assert.equal(artifact.target.src_path, source);
  assert.equal(artifact.fresh, false, "the relevant rustc invocation must actually execute");
  assert.deepEqual(artifact.features.toSorted(), expected.features);
  assert.equal(artifact.profile.opt_level, profile === "release" ? "3" : "0");
  assert.equal(artifact.profile.debuginfo, 0);
  assert.equal(artifact.profile.debug_assertions, profile !== "release");
  assert.equal(artifact.profile.overflow_checks, profile !== "release");
  for (const filename of artifact.filenames) {
    assert.ok(path.relative(target, filename).startsWith(`${profile}/`));
    assert.ok(fs.statSync(filename).isFile());
  }
  if (expected.kind === "bin") {
    assert.equal(artifact.executable, binary);
    assert.ok(artifact.filenames.includes(binary));
  }
  return {
    ...artifact,
    manifestSha256: sha256(fs.readFileSync(manifest)),
    sourceSha256: sha256(fs.readFileSync(source)),
  };
});
const binarySha256 = sha256(fs.readFileSync(binary));
const launchBinary = path.join(root, "target/ci/vize");
if (binary !== launchBinary) {
  fs.mkdirSync(path.dirname(launchBinary), { recursive: true });
  fs.copyFileSync(binary, launchBinary);
}
assert.equal(sha256(fs.readFileSync(launchBinary)), binarySha256);
if (side === "after") {
  const previous = JSON.parse(
    fs.readFileSync(path.join(output, "before-cargo-custody.json"), "utf8"),
  );
  assert.deepEqual(
    toolchain,
    previous.toolchain,
    "both literal sources need the same actual compiler",
  );
  assert.deepEqual(
    artifacts.map((entry) => entry.profile),
    previous.artifacts.map((entry: { profile: unknown }) => entry.profile),
  );
  assert.deepEqual(
    artifacts.map((entry) => entry.features),
    previous.artifacts.map((entry: { features: unknown }) => entry.features),
  );
}
const receiptPath = writeBuildReceipt(
  root,
  profile === "release" ? SHIPPING_BUILD_RECIPE : BUILD_RECIPE,
);
const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
assert.equal(receipt.binarySha256, binarySha256);
assert.equal(receipt.sourceRevision, side === "before" ? binding.baseline : binding.head);
assert.deepEqual(
  sourceIdentity(root),
  sourceBefore,
  "source/locks must not drift during the build",
);
fs.copyFileSync(receiptPath, path.join(output, `${side}-build.json`));
fs.writeFileSync(
  path.join(output, `${side}-cargo-custody.json`),
  `${JSON.stringify({ side, source: sourceIdentity(root), toolchain, profile, target, clean, build, artifacts, localArtifacts, binary, binarySha256, cliVersion: receipt.cliVersion, launchBinary, changedDependencies, dependencyArtifacts: binding.authority === "root-frozen-release-cut" ? "The complete selected profile is cleaned identically on both sides; every linked local compiler artifact is attested fresh. Other profile/toolchain artifacts are not attested by this receipt." : "Other unchanged dependencies reuse the existing Cargo target cache; the four required Canon/Maestro/CLI and any changed qualified dependency artifacts are attested fresh." }, null, 2)}\n`,
);

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { readCiRequest } from "./formatter-history-ci-request.mjs";
import {
  bytes,
  hash,
  emittedNativeArtifact,
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  nativeHistorySource,
  nativeHistoryBuildEnvironment,
  validateNativeHistoryEnvironment,
  napiToolchain,
} from "./formatter-history-build.mts";

export const ciBuildCommand = [
  "napi",
  "build",
  "--platform",
  "--profile",
  "ci",
  "--manifest-path",
  "../../crates/vize_vitrine/Cargo.toml",
  "-p",
  "vize_vitrine",
  "--features",
  "napi,legacy",
  "--output-dir",
  ".",
  "--",
  "--message-format=json-render-diagnostics",
  "--locked",
];
export function validateCiBuildCommand(command) {
  assert.deepEqual(command, ciBuildCommand);
}
const generatedPaths = ["npm/native/index.js", "npm/native/index.d.ts"];
const emptyHash = createHash("sha256").update(Buffer.alloc(0)).digest("hex");
const git = (directory, ...args) =>
  execFileSync("git", args, { cwd: directory, maxBuffer: 64 * 1024 * 1024 });
function generatedBytes(nativeDir) {
  const root = path.resolve(nativeDir, "../..");
  return Object.fromEntries(
    generatedPaths.map((file) => [file, bytes(fs.readFileSync(path.join(root, file)))]),
  );
}
export function ciSourceIdentity(nativeDir) {
  const root = path.resolve(nativeDir, "../..");
  const changed = git(root, "diff", "--name-only", "HEAD")
    .toString()
    .trim()
    .split("\n")
    .filter(Boolean);
  for (const file of changed)
    assert(generatedPaths.includes(file), `authored source changed: ${file}`);
  assert.equal(
    git(
      root,
      "ls-files",
      "--others",
      "--exclude-standard",
      "--",
      "davinci",
      "crates",
      "npm/native",
      "npm/builder/vite/src",
      "tests/differential",
      "tests/_fixtures/differential/formatter-history",
    ).length,
    0,
  );
  return {
    head: git(root, "rev-parse", "HEAD").toString().trim(),
    tree: git(root, "rev-parse", "HEAD^{tree}").toString().trim(),
    workingDiff: hash(git(root, "diff", "--binary", "HEAD")),
    cargoLock: hash(fs.readFileSync(path.join(root, "Cargo.lock"))),
    pnpmLock: hash(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))),
    vitrineSourceTree: git(root, "rev-parse", "HEAD:crates/vize_vitrine/src").toString().trim(),
    glyphSourceTree: git(root, "rev-parse", "HEAD:crates/vize_glyph/src").toString().trim(),
  };
}
export function validateCiProfile(profile) {
  assert.equal(profile.test, false);
  assert.equal(profile.opt_level, "0");
  assert(profile.debuginfo === null || profile.debuginfo === 0);
  assert.equal(profile.debug_assertions, true);
  assert.equal(profile.overflow_checks, true);
}
export function beforeCiBuild(nativeDir) {
  const source = nativeHistorySource(nativeDir);
  return {
    source,
    request: readCiRequest(nativeDir, source),
    environment: nativeHistoryBuildEnvironment(),
    generated: generatedBytes(nativeDir),
  };
}
function processPacket(command, result) {
  return {
    command,
    stdout: bytes(result.stdout ?? Buffer.alloc(0)),
    stderr: bytes(result.stderr ?? Buffer.alloc(0)),
    exitStatus: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  };
}
function validateProcess(process) {
  assert.equal(process.exitStatus, 0);
  assert.equal(process.signal, null);
  assert.equal(process.error, null);
}
function toolchain(nativeDir) {
  return {
    rustc: execFileSync("rustc", ["-Vv"]).toString(),
    cargo: execFileSync(process.env.CARGO ?? "cargo", ["-V"]).toString(),
    node: process.version,
    napi: napiToolchain(nativeDir),
  };
}
export function preserveCiBuildAttempt(nativeDir, before, build, sync = null) {
  const directory = nativeHistoryDirectory(nativeDir);
  fs.mkdirSync(directory, { recursive: true });
  for (const [label, process] of [
    ["cargo", build],
    ["sync", sync],
  ]) {
    if (!process) continue;
    for (const stream of ["stdout", "stderr"])
      fs.writeFileSync(
        path.join(directory, `${label}.${stream}.bin`),
        Buffer.from(process[stream].base64, "base64"),
      );
  }
  const diff = git(nativeDir, "diff", "--binary", "HEAD");
  fs.writeFileSync(path.join(directory, "generated-source.diff"), diff);
  fs.writeFileSync(
    path.join(directory, "ci-build-attempt.json"),
    JSON.stringify({
      kind: "actual process custody before admission; no qualification credit",
      before,
      process: build,
      synchronize: sync,
      generatedAfter: generatedBytes(nativeDir),
      sourceDiff: bytes(diff),
    }) + "\n",
  );
}
export function captureCiBuild(nativeDir, before, build, sync) {
  validateCiBuildCommand(build.command);
  validateProcess(build);
  validateProcess(sync);
  const directory = nativeHistoryDirectory(nativeDir);
  fs.mkdirSync(directory, { recursive: true });
  for (const [label, process] of [
    ["cargo", build],
    ["sync", sync],
  ])
    for (const stream of ["stdout", "stderr"])
      fs.writeFileSync(
        path.join(directory, `${label}.${stream}.bin`),
        Buffer.from(process[stream].base64, "base64"),
      );
  const source = ciSourceIdentity(nativeDir);
  assert.deepEqual({ ...source, workingDiff: emptyHash }, before.source);
  const emitted = emittedNativeArtifact(Buffer.from(build.stdout.base64, "base64"));
  validateCiProfile(emitted.cargo.profile);
  assert.equal(
    fs.realpathSync(emitted.cargo.target.src_path),
    fs.realpathSync(path.join(nativeDir, "../../crates/vize_vitrine/src/lib.rs")),
  );
  const names = fs.readdirSync(nativeDir).filter((name) => /^vize-vitrine\..*\.node$/.test(name));
  assert.equal(names.length, 1);
  const addon = fs.readFileSync(path.join(nativeDir, names[0]));
  assert(
    addon.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])),
    "actual ci addon must be ELF",
  );
  assert(
    addon.equals(fs.readFileSync(emitted.filename)),
    "ci addon differs from actual Cargo artifact",
  );
  const frozen = path.join(directory, "formatter-observe.node");
  fs.writeFileSync(frozen, addon, { flag: "wx" });
  const diff = git(nativeDir, "diff", "--binary", "HEAD");
  fs.writeFileSync(path.join(directory, "generated-source.diff"), diff);
  const receipt = {
    schema: "vize.public-native-formatter-ci-build",
    version: 1,
    request: before.request,
    sourceBefore: before.source,
    source,
    environment: validateNativeHistoryEnvironment({
      before: before.environment,
      after: nativeHistoryBuildEnvironment(),
    }),
    process: build,
    synchronize: sync,
    generatedSource: {
      before: before.generated,
      after: generatedBytes(nativeDir),
      diff: bytes(diff),
    },
    emitted,
    generated: { name: names[0], sha256: hash(addon) },
    frozen: { path: frozen, sha256: hash(addon) },
    toolchain: toolchain(nativeDir),
  };
  fs.writeFileSync(nativeHistoryReceipt(nativeDir), JSON.stringify(receipt) + "\n");
  return receipt;
}
export const captureCiProcess = processPacket;
export function validateCiStreamCustody(nativeDir, receipt) {
  for (const [label, process] of [
    ["cargo", receipt.process],
    ["sync", receipt.synchronize],
  ])
    for (const stream of ["stdout", "stderr"])
      assert.deepEqual(
        process[stream],
        bytes(
          fs.readFileSync(path.join(nativeHistoryDirectory(nativeDir), `${label}.${stream}.bin`)),
        ),
      );
}
export function validateCiArtifactCopies(nativeDir, receipt) {
  const names = fs.readdirSync(nativeDir).filter((name) => /^vize-vitrine\..*\.node$/.test(name));
  assert.equal(names.length, 1);
  assert.equal(receipt.generated.name, names[0]);
  const artifact = fs.readFileSync(receipt.emitted.filename);
  assert(
    artifact.length >= 64 && artifact.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])),
  );
  assert.equal(hash(artifact), receipt.generated.sha256);
  assert(fs.readFileSync(path.join(nativeDir, receipt.generated.name)).equals(artifact));
  assert.equal(
    fs.realpathSync(receipt.frozen.path),
    fs.realpathSync(path.join(nativeHistoryDirectory(nativeDir), "formatter-observe.node")),
  );
  assert(fs.readFileSync(receipt.frozen.path).equals(artifact));
  assert.equal(receipt.frozen.sha256, receipt.generated.sha256);
}
export function validateCiBuild(nativeDir, receipt) {
  assert.equal(receipt.schema, "vize.public-native-formatter-ci-build");
  assert.equal(receipt.version, 1);
  assert.deepEqual(receipt.request, readCiRequest(nativeDir, receipt.sourceBefore));
  validateCiBuildCommand(receipt.process.command);
  validateProcess(receipt.process);
  validateProcess(receipt.synchronize);
  assert.deepEqual(receipt.synchronize.command, [
    process.execPath,
    path.join(nativeDir, "scripts/sync-entrypoint.mjs"),
  ]);
  assert.deepEqual(receipt.source, ciSourceIdentity(nativeDir));
  assert.deepEqual(receipt.sourceBefore, { ...receipt.source, workingDiff: emptyHash });
  assert.deepEqual(Object.keys(receipt.generatedSource.before), generatedPaths);
  const root = path.resolve(nativeDir, "../..");
  for (const file of generatedPaths)
    assert.deepEqual(
      receipt.generatedSource.before[file],
      bytes(git(root, "show", `HEAD:${file}`)),
    );
  assert.deepEqual(receipt.generatedSource.after, generatedBytes(nativeDir));
  const diff = git(root, "diff", "--binary", "HEAD");
  assert.deepEqual(receipt.generatedSource.diff, bytes(diff));
  assert(
    fs
      .readFileSync(path.join(nativeHistoryDirectory(nativeDir), "generated-source.diff"))
      .equals(diff),
  );
  validateNativeHistoryEnvironment(receipt.environment);
  validateCiStreamCustody(nativeDir, receipt);
  const emitted = emittedNativeArtifact(Buffer.from(receipt.process.stdout.base64, "base64"));
  assert.deepEqual(receipt.emitted, emitted);
  validateCiProfile(emitted.cargo.profile);
  assert.equal(
    fs.realpathSync(emitted.cargo.target.src_path),
    fs.realpathSync(path.join(nativeDir, "../../crates/vize_vitrine/src/lib.rs")),
  );
  validateCiArtifactCopies(nativeDir, receipt);
  assert.deepEqual(receipt.toolchain, toolchain(nativeDir));
  return receipt;
}

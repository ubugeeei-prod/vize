import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

export const nativeHistoryDirectory = (nativeDir) =>
  path.join(nativeDir, ".artifacts/native/formatter-history");
export const nativeHistoryReceipt = (nativeDir) =>
  path.join(nativeHistoryDirectory(nativeDir), "build-receipt.json");
export const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const bytes = (value) => ({ base64: value.toString("base64"), sha256: hash(value) });

const buildEnvironmentKeys = [
  "CARGO",
  "CARGO_TARGET_DIR",
  "CARGO_BUILD_TARGET",
  "CARGO_BUILD_JOBS",
  "RUSTFLAGS",
  "CARGO_ENCODED_RUSTFLAGS",
  "CC",
  "CXX",
  "SDKROOT",
];
export function nativeHistoryBuildEnvironment() {
  return Object.fromEntries(
    buildEnvironmentKeys.map((key) => [key, globalThis.process.env[key] ?? null]),
  );
}
export function validateNativeHistoryEnvironment(environment) {
  assert.deepEqual(Object.keys(environment), ["before", "after"]);
  for (const observed of [environment.before, environment.after]) {
    assert.deepEqual(Object.keys(observed), buildEnvironmentKeys);
    for (const value of Object.values(observed))
      assert(value === null || typeof value === "string");
  }
  assert.deepEqual(
    environment.after,
    environment.before,
    "build environment changed during the actual build",
  );
  return environment;
}
function cargoVersion() {
  return execFileSync(globalThis.process.env.CARGO ?? "cargo", ["-V"]).toString();
}

function napiToolchain(nativeDir) {
  const entry = createRequire(path.join(nativeDir, "package.json")).resolve("@napi-rs/cli");
  const packageRoot = path.resolve(path.dirname(entry), "..");
  const metadata = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json")));
  return {
    version: metadata.version,
    cliSha256: hash(fs.readFileSync(path.join(packageRoot, "dist/cli.js"))),
  };
}

export function nativeHistorySource(nativeDir) {
  const root = path.resolve(nativeDir, "../..");
  const git = (...args) => execFileSync("git", args, { cwd: root, maxBuffer: 64 * 1024 * 1024 });
  const workingDiff = git("diff", "--binary", "HEAD");
  assert.equal(workingDiff.length, 0, "public native capture requires committed clean source");
  const untracked = git(
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
  );
  assert.equal(untracked.length, 0, "untracked native observer/source cannot qualify");
  return {
    head: git("rev-parse", "HEAD").toString().trim(),
    tree: git("rev-parse", "HEAD^{tree}").toString().trim(),
    workingDiff: hash(workingDiff),
    cargoLock: hash(fs.readFileSync(path.join(root, "Cargo.lock"))),
    pnpmLock: hash(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))),
    vitrineSourceTree: git("rev-parse", "HEAD:crates/vize_vitrine/src").toString().trim(),
    glyphSourceTree: git("rev-parse", "HEAD:crates/vize_glyph/src").toString().trim(),
  };
}

export function emittedNativeArtifact(stdout) {
  const messages = stdout
    .toString()
    .split("\n")
    .flatMap((line) => {
      try {
        return [JSON.parse(line)];
      } catch {
        return [];
      }
    });
  const finished = messages.filter(({ reason }) => reason === "build-finished");
  assert.equal(finished.length, 1, "missing genuine Cargo completion");
  assert.equal(finished[0].success, true);
  const artifacts = messages.filter(
    ({ reason, target }) =>
      reason === "compiler-artifact" &&
      target?.name === "vize_vitrine" &&
      target.kind.includes("cdylib"),
  );
  assert.equal(artifacts.length, 1, "ambiguous or absent actual native artifact");
  const artifact = artifacts[0];
  assert.equal(artifact.profile.test, false);
  assert(artifact.features.includes("napi") && artifact.features.includes("legacy"));
  const candidates = artifact.filenames.filter((file) => /\.(so|dylib|dll)$/.test(file));
  assert.equal(candidates.length, 1, "absent emitted shared library");
  return { cargo: artifact, filename: candidates[0] };
}

export function captureNativeHistoryBuild(nativeDir, before, command, result) {
  const directory = nativeHistoryDirectory(nativeDir);
  fs.mkdirSync(directory, { recursive: true });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  fs.writeFileSync(path.join(directory, "cargo.stdout.bin"), stdout);
  fs.writeFileSync(path.join(directory, "cargo.stderr.bin"), stderr);
  const process = {
    command,
    stdout: bytes(stdout),
    stderr: bytes(stderr),
    exitStatus: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  };
  fs.writeFileSync(path.join(directory, "build-process.json"), JSON.stringify(process));
  assert.equal(process.exitStatus, 0);
  assert.equal(process.signal, null);
  assert.equal(process.error, null);
  assert.deepEqual(
    nativeHistorySource(nativeDir),
    before.source,
    "source changed during native build",
  );
  const environment = validateNativeHistoryEnvironment({
    before: before.environment,
    after: nativeHistoryBuildEnvironment(),
  });
  const emitted = emittedNativeArtifact(stdout);
  const generated = fs
    .readdirSync(path.join(nativeDir, ".artifacts/native"))
    .filter((name) => name.startsWith("vize-vitrine.") && name.endsWith(".node"));
  assert.equal(generated.length, 1);
  assert.equal(
    fs.realpathSync(emitted.cargo.target.src_path),
    fs.realpathSync(path.join(nativeDir, "../../crates/vize_vitrine/src/lib.rs")),
  );
  const emittedBytes = fs.readFileSync(emitted.filename);
  const generatedBytes = fs.readFileSync(path.join(nativeDir, ".artifacts/native", generated[0]));
  assert(generatedBytes.equals(emittedBytes), "generated addon differs from actual Cargo artifact");
  const frozen = path.join(directory, "formatter-observe.node");
  fs.writeFileSync(frozen, generatedBytes, { flag: "wx" });
  const receipt = {
    schema: "vize.public-native-formatter-build",
    version: 1,
    source: before.source,
    environment,
    process,
    emitted,
    generated: { name: generated[0], sha256: hash(generatedBytes) },
    frozen: { path: frozen, sha256: hash(generatedBytes) },
    toolchain: {
      rustc: execFileSync("rustc", ["-Vv"]).toString(),
      cargo: cargoVersion(),
      node: globalThis.process.version,
      napi: napiToolchain(nativeDir),
    },
  };
  fs.writeFileSync(nativeHistoryReceipt(nativeDir), `${JSON.stringify(receipt)}\n`);
  return receipt;
}

export function validateNativeHistoryBuild(nativeDir, receipt) {
  assert.equal(receipt.schema, "vize.public-native-formatter-build");
  assert.equal(receipt.version, 1);
  assert.deepEqual(receipt.source, nativeHistorySource(nativeDir));
  validateNativeHistoryEnvironment(receipt.environment);
  assert.deepEqual(receipt.process.command, [
    "pnpm",
    "exec",
    "napi",
    "build",
    "--platform",
    "--manifest-path",
    "../../crates/vize_vitrine/Cargo.toml",
    "-p",
    "vize_vitrine",
    "--features",
    "napi,legacy",
    "--output-dir",
    path.join(nativeDir, ".artifacts/native"),
    "--no-js",
    "--",
    "--message-format=json-render-diagnostics",
    "--locked",
  ]);
  assert.equal(receipt.process.exitStatus, 0);
  assert.equal(receipt.process.signal, null);
  assert.equal(receipt.process.error, null);
  for (const stream of ["stdout", "stderr"]) {
    const raw = fs.readFileSync(
      path.join(nativeHistoryDirectory(nativeDir), `cargo.${stream}.bin`),
    );
    assert.deepEqual(receipt.process[stream], bytes(raw));
  }
  assert.deepEqual(
    receipt.emitted,
    emittedNativeArtifact(Buffer.from(receipt.process.stdout.base64, "base64")),
  );
  assert.equal(
    fs.realpathSync(receipt.emitted.cargo.target.src_path),
    fs.realpathSync(path.join(nativeDir, "../../crates/vize_vitrine/src/lib.rs")),
  );
  assert.equal(hash(fs.readFileSync(receipt.emitted.filename)), receipt.generated.sha256);
  assert.equal(
    fs.realpathSync(receipt.frozen.path),
    fs.realpathSync(path.join(nativeHistoryDirectory(nativeDir), "formatter-observe.node")),
  );
  assert.equal(hash(fs.readFileSync(receipt.frozen.path)), receipt.frozen.sha256);
  assert.equal(receipt.frozen.sha256, receipt.generated.sha256);
  assert.equal(receipt.toolchain.rustc, execFileSync("rustc", ["-Vv"]).toString());
  assert.equal(receipt.toolchain.cargo, cargoVersion());
  assert.equal(receipt.toolchain.node, globalThis.process.version);
  assert.deepEqual(receipt.toolchain.napi, napiToolchain(nativeDir));
  return receipt;
}

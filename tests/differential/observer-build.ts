import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "./harness.mjs";

export interface ObserverSpec {
  product: string;
  packageName: string;
  exampleName: string;
  sourcePath: string;
  probes: readonly (readonly string[])[];
  expectedFeatures?: readonly string[];
}

type BuildOptions = {
  spec: ObserverSpec;
  repoRoot: string;
  targetDir: string;
  evidenceDir: string;
  profile?: "dev" | "ci";
  offline?: boolean;
};

function git(root: string, args: string[]) {
  const result = spawnSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trimEnd();
}

function validateSpec(spec: ObserverSpec) {
  assert.match(spec.product, /^[a-z][a-z0-9-]*$/);
  assert.match(spec.packageName, /^[a-z][a-z0-9_]*$/);
  assert.match(spec.exampleName, /^[a-z][a-z0-9_]*$/);
  assert(
    [
      `crates/${spec.packageName}/examples/${spec.exampleName}.rs`,
      `crates/${spec.packageName}/examples/${spec.exampleName}/main.rs`,
    ].includes(spec.sourcePath),
  );
  assert(Array.isArray(spec.probes) && spec.probes.length > 0);
  for (const probe of spec.probes) {
    assert(Array.isArray(probe) && probe.every((arg) => typeof arg === "string"));
  }
  assert((spec.expectedFeatures ?? []).every((feature) => /^[a-z][a-z0-9_-]*$/.test(feature)));
}

export function observerBuildArgs(spec: ObserverSpec, profile: string, offline: boolean) {
  validateSpec(spec);
  assert(["dev", "ci"].includes(profile), "unknown observer build profile");
  assert.equal(typeof offline, "boolean");
  return [
    "build",
    "--locked",
    ...(offline ? ["--offline"] : []),
    "--profile",
    profile,
    "-p",
    spec.packageName,
    "--example",
    spec.exampleName,
    "--message-format=json-render-diagnostics",
  ];
}

export function productObserverSourceIdentity(repoRoot: string, spec: ObserverSpec) {
  validateSpec(spec);
  const scopes = [
    "Cargo.lock",
    "Cargo.toml",
    ".cargo",
    "rust-toolchain",
    "rust-toolchain.toml",
    "crates",
    "davinci",
    "tests",
    "tools/benchmarks/crates",
  ];
  const untracked = git(repoRoot, [
    "ls-files",
    "--others",
    "--exclude-standard",
    "-z",
    "--",
    ...scopes,
  ])
    .split("\0")
    .filter((file) => /(?:\.rs|Cargo\.toml|Cargo\.lock)$/.test(file));
  assert.deepEqual(untracked, [], "untracked Rust/Cargo inputs cannot enter an observation");
  assert.equal(
    git(repoRoot, ["diff", "--name-only", "HEAD", "--", ...scopes]),
    "",
    "source-built observation requires committed Rust/Cargo inputs",
  );
  assert.equal(
    git(repoRoot, ["ls-files", "--error-unmatch", "--", spec.sourcePath]),
    spec.sourcePath,
  );
  return {
    sourceRevision: git(repoRoot, ["rev-parse", "HEAD"]),
    sourceTree: git(repoRoot, ["rev-parse", "HEAD^{tree}"]),
    productSourceTree: git(repoRoot, ["rev-parse", `HEAD:crates/${spec.packageName}/src`]),
    cargoLockSha256: sha256(fs.readFileSync(path.join(repoRoot, "Cargo.lock"))),
    observerSourceSha256: sha256(fs.readFileSync(path.join(repoRoot, spec.sourcePath))),
  };
}

export function observerToolchain(program: string, root: string) {
  const result = spawnSync(program, ["--version"], { cwd: root, encoding: "utf8" });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function selectObserverArtifact(stdout: Buffer, spec: ObserverSpec, repoRoot: string) {
  validateSpec(spec);
  const messages = stdout
    .toString()
    .split("\n")
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    messages.filter((item) => item.reason === "build-finished").map((item) => item.success),
    [true],
    "successful Cargo build-finished evidence is required",
  );
  const artifacts = messages.filter(
    (item) =>
      item.reason === "compiler-artifact" &&
      item.target.name === spec.exampleName &&
      item.target.kind.includes("example") &&
      item.executable,
  );
  assert.equal(artifacts.length, 1, "exactly one selected observer artifact is required");
  const selected = artifacts[0];
  validateWorkspacePackageId(selected.package_id, spec, repoRoot);
  assert.equal(
    fs.realpathSync(selected.target.src_path),
    fs.realpathSync(path.join(repoRoot, spec.sourcePath)),
  );
  assert.deepEqual(selected.target.kind, ["example"]);
  assert.equal(selected.profile.test, false);
  assert.deepEqual(selected.features, spec.expectedFeatures ?? []);
  return selected;
}

function validateWorkspacePackageId(packageId: string, spec: ObserverSpec, repoRoot: string) {
  assert(packageId.startsWith("path+file:"), "observer must be this workspace path package");
  const url = new URL(packageId.slice("path+".length));
  const version = url.hash.slice(1);
  const bareVersion = version.startsWith(`${spec.packageName}@`)
    ? version.slice(spec.packageName.length + 1)
    : version;
  assert.match(
    bareVersion,
    /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.+-]+)?$/,
    "unexpected Cargo package name/version",
  );
  url.hash = "";
  assert.equal(
    fs.realpathSync(fileURLToPath(url)),
    fs.realpathSync(path.join(repoRoot, "crates", spec.packageName)),
    "Cargo workspace package path differs",
  );
}

// Only a successful Cargo JSON artifact from this invocation is usable. An old
// executable cannot turn a failed build into a successful observation.
export function compileProductObserver({
  spec,
  repoRoot,
  targetDir,
  evidenceDir,
  profile = "dev",
  offline = false,
}: BuildOptions) {
  const source = productObserverSourceIdentity(repoRoot, spec);
  assert(path.isAbsolute(targetDir) && path.isAbsolute(evidenceDir));
  fs.mkdirSync(evidenceDir, { recursive: true });
  const argv = observerBuildArgs(spec, profile, offline);
  const result = spawnSync("cargo", argv, {
    cwd: repoRoot,
    env: { ...process.env, CARGO_TARGET_DIR: targetDir, CARGO_INCREMENTAL: "0" },
    timeout: 300_000,
    maxBuffer: 32 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  const logs = {
    stdout: { path: path.join(evidenceDir, "cargo.jsonl"), sha256: sha256(stdout) },
    stderr: { path: path.join(evidenceDir, "cargo.stderr.txt"), sha256: sha256(stderr) },
  };
  fs.writeFileSync(logs.stdout.path, stdout);
  fs.writeFileSync(logs.stderr.path, stderr);
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, stderr.toString());
  const selected = selectObserverArtifact(stdout, spec, repoRoot);
  assert.deepEqual(
    productObserverSourceIdentity(repoRoot, spec),
    source,
    "source changed during build",
  );
  const binaryPath = path.join(
    evidenceDir,
    `${spec.exampleName}${process.platform === "win32" ? ".exe" : ""}`,
  );
  fs.copyFileSync(selected.executable, binaryPath);
  fs.chmodSync(binaryPath, 0o755);
  return {
    binaryPath,
    command: ["cargo", ...argv],
    logs,
    exitStatus: result.status,
    artifact: {
      packageId: selected.package_id,
      target: selected.target,
      profile: selected.profile,
      features: selected.features,
      executable: binaryPath,
      cargoExecutable: selected.executable,
      sha256: sha256(fs.readFileSync(binaryPath)),
    },
  };
}

function observeProbe(binaryPath: string, argv: readonly string[]) {
  const result = spawnSync(binaryPath, [...argv], { timeout: 30_000, maxBuffer: 4 * 1024 * 1024 });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr?.toString());
  assert.equal(result.stderr.length, 0);
  return {
    argv,
    exitStatus: result.status,
    stdoutBase64: result.stdout.toString("base64"),
    sha256: sha256(result.stdout),
  };
}

export function validateProductObserverReceipt(
  receipt: any,
  { spec, repoRoot, binaryPath }: { spec: ObserverSpec; repoRoot: string; binaryPath: string },
) {
  assert.equal(receipt.schema, "vize.differential.observer-build");
  assert.equal(receipt.version, 1);
  assert.deepEqual(receipt.spec, spec);
  assert.deepEqual(receipt.source, productObserverSourceIdentity(repoRoot, spec));
  assert.deepEqual(receipt.command, [
    "cargo",
    ...observerBuildArgs(spec, receipt.profileName, receipt.offline),
  ]);
  assert.equal(receipt.exitStatus, 0);
  assert.equal(receipt.artifact.target.name, spec.exampleName);
  assert.deepEqual(receipt.artifact.target.kind, ["example"]);
  assert.equal(
    fs.realpathSync(receipt.artifact.target.src_path),
    fs.realpathSync(path.join(repoRoot, spec.sourcePath)),
  );
  validateWorkspacePackageId(receipt.artifact.packageId, spec, repoRoot);
  assert.equal(receipt.artifact.profile.test, false);
  assert.deepEqual(receipt.artifact.features, spec.expectedFeatures ?? []);
  assert.equal(fs.realpathSync(receipt.artifact.executable), fs.realpathSync(binaryPath));
  assert.equal(receipt.artifact.sha256, sha256(fs.readFileSync(binaryPath)));
  assert.deepEqual(Object.keys(receipt.logs).sort(), ["stderr", "stdout"]);
  for (const log of Object.values(receipt.logs) as { path: string; sha256: string }[])
    assert.equal(sha256(fs.readFileSync(log.path)), log.sha256, "Cargo evidence log drift");
  const selected = selectObserverArtifact(
    fs.readFileSync(receipt.logs.stdout.path),
    spec,
    repoRoot,
  );
  assert.deepEqual(receipt.artifact.target, selected.target);
  assert.deepEqual(receipt.artifact.profile, selected.profile);
  assert.deepEqual(receipt.artifact.features, selected.features);
  assert.equal(receipt.artifact.packageId, selected.package_id);
  assert.equal(receipt.artifact.cargoExecutable, selected.executable);
  assert.deepEqual(receipt.toolchain, {
    rustc: observerToolchain("rustc", repoRoot),
    cargo: observerToolchain("cargo", repoRoot),
  });
  assert.deepEqual(
    receipt.probes,
    spec.probes.map((argv) => observeProbe(binaryPath, argv)),
  );
  return receipt;
}

export function buildProductObserver(options: BuildOptions) {
  const { spec, repoRoot, profile = "dev", offline = false } = options;
  const source = productObserverSourceIdentity(repoRoot, spec);
  const built = compileProductObserver(options);
  const receipt = {
    schema: "vize.differential.observer-build",
    version: 1,
    spec,
    source,
    profileName: profile,
    offline,
    ...built,
    toolchain: {
      rustc: observerToolchain("rustc", repoRoot),
      cargo: observerToolchain("cargo", repoRoot),
    },
    probes: spec.probes.map((argv) => observeProbe(built.binaryPath, argv)),
  };
  assert.deepEqual(
    productObserverSourceIdentity(repoRoot, spec),
    source,
    "source changed during build",
  );
  validateProductObserverReceipt(receipt, { spec, repoRoot, binaryPath: built.binaryPath });
  fs.writeFileSync(
    path.join(options.evidenceDir, "build-receipt.json"),
    `${JSON.stringify(receipt, null, 2)}\n`,
  );
  return { binaryPath: built.binaryPath, receipt };
}

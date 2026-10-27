/** Build custody: each immutable source/feature role owns a fresh Cargo target. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

const hash = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
const flags = [
  "build",
  "--locked",
  "--profile",
  "ci-opt",
  "-p",
  "vize",
  "--message-format=json-render-diagnostics",
  "--config",
  'profile.ci-opt.inherits="release"',
  "--config",
  'profile.ci-opt.lto="thin"',
  "--config",
  "profile.ci-opt.codegen-units=16",
  "--config",
  'profile.ci-opt.package.vize.strip="symbols"',
];

function profileSource(root: string) {
  const path = join(root, "Cargo.toml");
  if (!existsSync(path)) return null;
  let selected = false;
  return readFileSync(path, "utf8")
    .split("\n")
    .filter((line) => {
      if (line.startsWith("[")) selected = /^\[profile\.(?:release|ci-opt)(?:\.|\])/u.test(line);
      return selected;
    })
    .join("\n");
}

function output(executable: string, args: string[], cwd: string) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function buildPrPerformanceBinary(options: {
  sourceRoot: string;
  targetDir: string;
  binary: string;
  receipt: string;
  role: string;
  /** Test-only executable injection; Actions uses cargo from its pinned toolchain. */
  cargoExecutable?: string;
}) {
  const sourceRoot = resolve(options.sourceRoot);
  const targetDir = resolve(options.targetDir);
  const binary = resolve(options.binary);
  const receiptPath = resolve(options.receipt);
  assert.ok(!existsSync(targetDir), "each role requires a fresh, unshared Cargo target directory");
  assert.ok(!existsSync(binary) && !existsSync(receiptPath), "refuse to replace build evidence");
  const sourceSha = output("git", ["rev-parse", "HEAD"], sourceRoot);
  const treeSha = output("git", ["rev-parse", "HEAD^{tree}"], sourceRoot);
  assert.equal(
    output("git", ["status", "--porcelain", "--untracked-files=no"], sourceRoot),
    "",
    "only a clean immutable source may qualify a build",
  );
  mkdirSync(dirname(receiptPath), { recursive: true });
  const cargo = options.cargoExecutable ?? "cargo";
  const args = [...flags];
  const startedAt = new Date().toISOString();
  const result = spawnSync(cargo, args, {
    cwd: sourceRoot,
    env: { ...process.env, CARGO_TARGET_DIR: targetDir },
    encoding: "utf8",
    timeout: 30 * 60_000,
    maxBuffer: 128 * 1024 * 1024,
  });
  writeFileSync(receiptPath + ".stdout.jsonl", result.stdout ?? "");
  writeFileSync(receiptPath + ".stderr.txt", result.stderr ?? "");
  process.stderr.write(result.stderr ?? "");
  assert.equal(result.status, 0, `build failed; see ${receiptPath}.stderr.txt and .stdout.jsonl`);
  const artifacts = (result.stdout ?? "")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        row.target?.name === "vize" &&
        row.target.kind.includes("bin"),
    );
  assert.equal(artifacts.length, 1, "Cargo must identify the actual vize executable");
  const built = resolve(artifacts[0].executable);
  assert.ok(Array.isArray(artifacts[0].features), "Cargo must report CLI features");
  const features: string[] = artifacts[0].features;
  assert.equal(
    features.includes("profiling"),
    false,
    "Cargo feature role must match the measured binary",
  );
  assert.equal(artifacts[0].fresh, false, "the root executable must be built in its fresh target");
  assert.equal(
    artifacts[0].profile?.debug_assertions,
    false,
    "normal profile must disable debug assertions",
  );
  assert.equal(
    artifacts[0].profile?.test,
    false,
    "normal profile must not compile a test executable",
  );
  assert.equal(
    built,
    join(targetDir, "ci-opt", process.platform === "win32" ? "vize.exe" : "vize"),
  );
  assert.equal(
    output("git", ["rev-parse", "HEAD"], sourceRoot),
    sourceSha,
    "source changed during build",
  );
  assert.equal(output("git", ["status", "--porcelain", "--untracked-files=no"], sourceRoot), "");
  mkdirSync(dirname(binary), { recursive: true });
  if (built !== binary) {
    copyFileSync(built, binary);
    chmodSync(binary, statSync(built).mode & 0o777);
  }
  const receipt = {
    schemaVersion: 1,
    role: options.role,
    sourceSha,
    treeSha,
    manifests: Object.fromEntries(
      ["Cargo.toml", "Cargo.lock", ".cargo/config.toml", ".cargo/config"].map((name) => [
        name,
        existsSync(join(sourceRoot, name)) ? hash(join(sourceRoot, name)) : null,
      ]),
    ),
    sourceRoot,
    targetDir,
    targetStartedEmpty: true,
    binary,
    binarySha256: hash(binary),
    binaryBytes: statSync(binary).size,
    cargoArtifact: {
      executable: built,
      fresh: artifacts[0].fresh ?? null,
      packageId: artifacts[0].package_id ?? null,
      features,
    },
    compilerEnvironment: Object.fromEntries(
      [
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_TARGET",
        "CARGO_BUILD_BUILD_DIR",
        "CARGO_PROFILE_CI_OPT_LTO",
        "CARGO_PROFILE_CI_OPT_CODEGEN_UNITS",
        "CARGO_PROFILE_CI_OPT_STRIP",
        "CARGO_PROFILE_CI_OPT_DEBUG",
        "CARGO_PROFILE_CI_OPT_OPT_LEVEL",
      ].map((key) => [key, process.env[key] ?? null]),
    ),
    allocation: false,
    profile: {
      name: "ci-opt",
      source: profileSource(sourceRoot),
      artifact: artifacts[0].profile ?? null,
    },
    cargo: { executable: cargo, args, version: output(cargo, ["--version"], sourceRoot) },
    rustc: output(process.env.RUSTC ?? "rustc", ["-Vv"], sourceRoot),
    startedAt,
    finishedAt: new Date().toISOString(),
  };
  writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + "\n");
  console.log(JSON.stringify(receipt));
  return receipt;
}

/** Validate an individual build without rejecting legitimate identical code. */
export function verifyPrPerformanceBuildReceipt(
  path: string,
  expected: { role?: string; sourceSha?: string } = {},
) {
  const row = JSON.parse(readFileSync(path, "utf8"));
  assert.equal(row.schemaVersion, 1);
  assert.match(row.sourceSha, /^[a-f0-9]{40}$/u);
  assert.match(row.treeSha, /^[a-f0-9]{40}$/u);
  assert.equal(row.targetStartedEmpty, true);
  assert.equal(row.cargoArtifact.fresh, false);
  assert.ok(Array.isArray(row.cargoArtifact.features));
  assert.equal(row.cargoArtifact.features.includes("profiling"), false);
  assert.equal(row.allocation, false);
  assert.equal(row.profile.name, "ci-opt");
  assert.equal(row.profile.artifact.debug_assertions, false);
  assert.equal(row.profile.artifact.test, false);
  if (expected.role) assert.equal(row.role, expected.role);
  if (expected.sourceSha) assert.equal(row.sourceSha, expected.sourceSha);
  assert.equal(hash(row.binary), row.binarySha256, "built executable bytes changed");
  assert.equal(statSync(row.binary).size, row.binaryBytes);
  return row;
}

export function verifyPrPerformanceBuildPair(basePath: string, headPath: string) {
  const base = verifyPrPerformanceBuildReceipt(basePath, { role: "base" });
  const head = verifyPrPerformanceBuildReceipt(headPath, { role: "head" });
  assert.notEqual(base.targetDir, head.targetDir, "source roles must use distinct fresh targets");
  assert.deepEqual(base.cargo.args, head.cargo.args, "compiler arguments must match");
  assert.equal(base.cargo.version, head.cargo.version);
  assert.equal(base.rustc, head.rustc);
  assert.deepEqual(base.compilerEnvironment, head.compilerEnvironment);
  assert.deepEqual(base.profile.artifact, head.profile.artifact);
  assert.deepEqual(base.cargoArtifact.features, head.cargoArtifact.features);
  return { qualified: true, identicalBinary: base.binarySha256 === head.binarySha256, base, head };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      "source-root": { type: "string" },
      "target-dir": { type: "string" },
      binary: { type: "string" },
      receipt: { type: "string" },
      role: { type: "string" },
    },
  });
  assert.ok(
    values["source-root"] && values["target-dir"] && values.binary && values.receipt && values.role,
  );
  buildPrPerformanceBinary({
    sourceRoot: values["source-root"],
    targetDir: values["target-dir"],
    binary: values.binary,
    receipt: values.receipt,
    role: values.role,
  });
}

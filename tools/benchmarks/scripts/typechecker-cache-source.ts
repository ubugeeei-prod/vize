/** Fresh normal shipping-profile source CLI with actual Cargo/source identity; no root observer. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import {
  writeBuildReceipt,
  SHIPPING_BUILD_RECIPE,
} from "../../../tests/differential/build-receipt.ts";

const { values } = parseArgs({
  options: {
    "source-root": { type: "string" },
    "target-dir": { type: "string" },
    receipt: { type: "string" },
  },
});
assert.ok(values["source-root"] && values["target-dir"] && values.receipt);
const sourceRoot = fs.realpathSync(values["source-root"]);
const target = path.resolve(values["target-dir"]),
  destination = path.join(
    sourceRoot,
    "target/ci",
    process.platform === "win32" ? "vize.exe" : "vize",
  );
assert.ok(
  !fs.existsSync(target),
  "actual target must start absent, independently of all other roles",
);
assert.ok(!fs.existsSync(destination), "refuse to replace an unqualified staging binary");
const receipt = path.resolve(values.receipt);
fs.mkdirSync(path.dirname(receipt), { recursive: true });
const hash = (p: string) => createHash("sha256").update(fs.readFileSync(p)).digest("hex");
const git = (...args: string[]) => {
  const result = spawnSync("git", args, { cwd: sourceRoot, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
};
assert.equal(
  git("status", "--porcelain", "--untracked-files=no"),
  "",
  "source must start tracked-clean",
);
const sourceSha = git("rev-parse", "HEAD"),
  sourceTree = git("rev-parse", "HEAD^{tree}");
const args = [
  "build",
  "--locked",
  "--release",
  "-p",
  "vize",
  "--message-format=json-render-diagnostics",
  "--config",
  "profile.release.opt-level=3",
  "--config",
  'profile.release.lto="fat"',
  "--config",
  "profile.release.codegen-units=1",
  "--config",
  "profile.release.debug=0",
  "--config",
  'profile.release.package.vize.strip="symbols"',
  "--config",
  "profile.release.debug-assertions=false",
  "--config",
  "profile.release.overflow-checks=false",
  "--config",
  "profile.release.incremental=false",
  "--config",
  'profile.release.panic="unwind"',
];
const environment = {
  ...process.env,
  CARGO_TARGET_DIR: target,
  CARGO_INCREMENTAL: "0",
};
const compilerEnvironment = Object.fromEntries(
  Object.entries(environment).filter(
    ([key]) =>
      [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTUP_TOOLCHAIN",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_BUILD_RUSTC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_TARGET",
        "CARGO_INCREMENTAL",
      ].includes(key) ||
      key.startsWith("CARGO_PROFILE_RELEASE_") ||
      /^CARGO_TARGET_[A-Z0-9_]+_(RUSTFLAGS|LINKER)$/u.test(key),
  ),
);
const tool = (name: string, argv: string[]) => {
  const result = spawnSync(name, argv, {
    cwd: sourceRoot,
    env: environment,
    encoding: "utf8",
    timeout: 30_000,
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
};
const cargoVersion = tool("cargo", ["-Vv"]),
  rustcVersion = tool("rustc", ["-Vv"]);
const result = spawnSync("cargo", args, {
  cwd: sourceRoot,
  env: environment,
  encoding: "utf8",
  maxBuffer: 128 * 1024 * 1024,
  timeout: 1_200_000,
  killSignal: "SIGKILL",
});
fs.writeFileSync(receipt + ".stdout.jsonl", result.stdout ?? "");
fs.writeFileSync(receipt + ".stderr.txt", result.stderr ?? "");
assert.equal(result.error, undefined);
assert.equal(result.signal, null);
assert.equal(result.status, 0, "source build failed; inspect retained Cargo output");
const artifacts = (result.stdout ?? "").split("\n").flatMap((line) => {
  try {
    const row = JSON.parse(line);
    return row.reason === "compiler-artifact" &&
      row.target?.name === "vize" &&
      row.target?.kind?.includes("bin") &&
      row.executable
      ? [row]
      : [];
  } catch {
    return [];
  }
});
assert.equal(artifacts.length, 1, "actual executable must come from Cargo JSON");
const artifact = artifacts[0];
assert.equal(artifact.fresh, false, "this source executable must really compile");
assert.ok(Array.isArray(artifact.features) && !artifact.features.includes("profiling"));
assert.equal(artifact.profile.opt_level, "3");
assert.equal(artifact.profile.debuginfo, 0);
assert.equal(artifact.profile.debug_assertions, false);
assert.equal(artifact.profile.test, false);
const built = fs.realpathSync(artifact.executable);
const physicalTarget = fs.realpathSync(target);
const relativeBuilt = path.relative(physicalTarget, built);
assert.ok(
  relativeBuilt !== "" &&
    relativeBuilt !== ".." &&
    !relativeBuilt.startsWith(".." + path.sep) &&
    !path.isAbsolute(relativeBuilt),
  "Cargo executable escaped the physically resolved task target",
);
assert.equal(git("rev-parse", "HEAD"), sourceSha);
assert.equal(
  git("status", "--porcelain", "--untracked-files=no"),
  "",
  "build changed tracked source",
);
fs.mkdirSync(path.dirname(destination), { recursive: true });
fs.copyFileSync(built, destination);
fs.chmodSync(destination, fs.statSync(built).mode);
assert.equal(hash(built), hash(destination));
const differential = writeBuildReceipt(sourceRoot, SHIPPING_BUILD_RECIPE);
const probe = spawnSync(destination, ["--version"], { encoding: "utf8", timeout: 30_000 });
assert.equal(probe.status, 0);
const canonical = JSON.parse(fs.readFileSync(differential, "utf8"));
assert.equal(canonical.sourceRevision, sourceSha);
assert.equal(canonical.binarySha256, hash(destination));
assert.equal(
  canonical.cliVersion,
  probe.stdout.trim(),
  "canonical identity must match the actual version probe",
);
const manifest = {
  schema: "vize.cold-source-build",
  version: 1,
  sourceSha,
  sourceTree,
  profile: "release",
  scope: "shipping fat LTO profile, no allocation instrumentation; compiler settings retained",
  sourceRoot,
  targetDir: target,
  targetStartedAbsent: true,
  binary: destination,
  binarySha256: hash(destination),
  binaryBytes: fs.statSync(destination).size,
  differentialReceipt: differential,
  differentialReceiptSha256: hash(differential),
  actualCargoExecutable: built,
  cargoArtifact: artifact,
  compilerEnvironment,
  cargo: { executable: "cargo", args, version: cargoVersion },
  rustc: rustcVersion,
  manifests: Object.fromEntries(
    ["Cargo.toml", "Cargo.lock", ".cargo/config", ".cargo/config.toml"]
      .filter((name) => fs.existsSync(path.join(sourceRoot, name)))
      .map((name) => [name, hash(path.join(sourceRoot, name))]),
  ),
  cliVersion: probe.stdout.trim(),
};
fs.writeFileSync(receipt, JSON.stringify(manifest, null, 2) + "\n");
console.log(
  JSON.stringify({
    sourceSha,
    binary: destination,
    binarySha256: manifest.binarySha256,
    profile: manifest.profile,
    bytes: manifest.binaryBytes,
  }),
);

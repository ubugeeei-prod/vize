import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, writeJson, fileIdentity, environment } from "../shared/files.ts";

import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, realpathSync } from "node:fs";
import { dirname, relative, isAbsolute } from "node:path";
import { executableArtifact } from "../shared/cargo-artifacts.ts";
const source = requiredEnv("SOURCE_ROOT"),
  role = requiredEnv("SOURCE_ROLE");
const root = join(requiredEnv("RUNNER_TEMP"), "instance-directives");
const log = join(root, "builds", role, "cargo.jsonl");
const artifact = executableArtifact(log, "vize", "bin");
const actual = realpathSync(artifact.executable),
  targetDir = realpathSync(requiredEnv("CARGO_TARGET_DIR"));
const rel = relative(targetDir, actual);
assert.ok(!isAbsolute(rel) && !rel.startsWith("../") && rel !== "..");
const binary = join(root, "binaries", role, "vize");
mkdirSync(dirname(binary), { recursive: true });
copyFileSync(actual, binary);
const git = (...args: string[]) =>
  execFileSync("git", ["-C", source, ...args], { encoding: "utf8" }).trim();
git("diff", "--exit-code");
writeJson(join(root, "builds", role, "identity.json"), {
  role,
  sourceSha: git("rev-parse", "HEAD"),
  sourceTree: git("rev-parse", "HEAD^{tree}"),
  targetDir,
  cargoArtifact: artifact,
  binary: fileIdentity(binary),
  manifest: fileIdentity(join(source, "Cargo.toml")),
  lock: fileIdentity(join(source, "Cargo.lock")),
  profile: "ci-opt",
  features: [],
  environment: environment([
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "CARGO_BUILD_JOBS",
    "CARGO_TARGET_DIR",
  ]),
});

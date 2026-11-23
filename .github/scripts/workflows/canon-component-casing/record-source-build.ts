import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, writeJson, fileIdentity } from "../shared/files.ts";

import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { executableArtifact } from "../shared/cargo-artifacts.ts";
const root = requiredEnv("EVIDENCE");
const cargoArtifact = executableArtifact(join(root, "build.jsonl"), "vize");
assert.equal(cargoArtifact.fresh, false);
const binary = cargoArtifact.executable;
const { sha256, bytes } = fileIdentity(binary);
const git = (...args: string[]) => execFileSync("git", args, { encoding: "utf8" }).trim();
const actualParents = git("show", "-s", "--format=%P", "HEAD").split(/\s+/u);
const pullRequestHead = requiredEnv("PR_HEAD_SHA");
assert.equal(actualParents.length, 2);
assert.equal(actualParents[1], pullRequestHead);
writeJson(join(root, "source-build.json"), {
  sourceRevision: git("rev-parse", "HEAD"),
  actualParents,
  pullRequestHead,
  binary,
  sha256,
  bytes,
  cargoArtifact,
  target: requiredEnv("CARGO_TARGET_DIR"),
  overlay: false,
  RUSTFLAGS: process.env.RUSTFLAGS ?? null,
  version: execFileSync(binary, ["--version"], { encoding: "utf8" }).trim(),
  timingUsable: false,
});
writeFileSync(join(root, "source-binary.path"), binary);

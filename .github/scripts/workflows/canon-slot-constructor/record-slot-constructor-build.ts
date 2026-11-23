import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile, writeJson, fileIdentity, environment } from "../shared/files.ts";

import { copyFileSync } from "node:fs";
import { executableArtifact } from "../shared/cargo-artifacts.ts";
const root = requiredEnv("SLOT_EVIDENCE");
const cargoArtifact = executableArtifact(join(root, "head-cargo.jsonl"), "vize");
assert.equal(cargoArtifact.fresh, false);
const destination = join(requiredEnv("RUNNER_TEMP"), "slot-head");
copyFileSync(cargoArtifact.executable, destination);
const { sha256, bytes } = fileIdentity(destination);
const parents = textFile(join(root, "product-parents.txt")).trim().split(/\s+/u);
const prHead = requiredEnv("PR_HEAD");
assert.equal(parents[1], prHead);
writeJson(join(root, "head-build.json"), {
  source: textFile(join(root, "product-source.txt")).trim(),
  parents,
  prHead,
  profile: "ci-opt",
  argv: [
    "cargo",
    "build",
    "--locked",
    "--profile",
    "ci-opt",
    "-p",
    "vize",
    "--bin",
    "vize",
    "--message-format=json",
  ],
  cargoArtifact,
  targetDirectory: requiredEnv("CARGO_TARGET_DIR"),
  sha256,
  bytes,
  overlay: false,
  timingComparison: false,
  environment: environment(["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL"]),
});

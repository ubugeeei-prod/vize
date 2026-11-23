import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile, writeJson, fileIdentity } from "../shared/files.ts";

import { writeFileSync, copyFileSync } from "node:fs";
import { executableArtifact } from "./cargo-artifacts.ts";

const [target, role] = process.argv.slice(2);
assert.ok(target && role);
const temp = requiredEnv("RUNNER_TEMP");
const anchor = target === "setup_binding_anchor_profile",
  normal = target === "vize";
const root = anchor ? join(temp, "binding-evidence") : requiredEnv("EVIDENCE");
const mode = normal ? "normal" : "profile";
const log = join(root, role + (anchor ? "-build.jsonl" : "-" + mode + "-build.jsonl"));
const artifact = executableArtifact(log, target);
if (!anchor) assert.equal(artifact.fresh, false);
const source = textFile(join(root, role + ".source")).trim();
const targetDirectory = join(
  temp,
  anchor ? "binding-" + role + "-target" : "component-" + role + "-" + mode + "-target",
);
if (normal) {
  const file = join(temp, "component-" + role + "-normal");
  copyFileSync(artifact.executable, file);
  const { bytes, sha256 } = fileIdentity(file);
  writeJson(join(root, role + "-normal-binary.json"), {
    source,
    file,
    bytes,
    sha256,
    fresh: artifact.fresh,
    cargoArtifact: artifact,
    targetDirectory,
    overlay: false,
  });
} else {
  const binary = artifact.executable;
  const { bytes, sha256 } = fileIdentity(binary);
  writeFileSync(join(root, role + (anchor ? ".binary" : ".profile-binary")), binary);
  const details = anchor
    ? {
        profile:
          "ci-opt; profiling allocator compiled; runtime-off and allocation windows separate",
      }
    : { cargoArtifact: artifact, commonOverlay: "profiling-overlays.json" };
  writeJson(join(root, role + (anchor ? "-binary.json" : "-profile-binary.json")), {
    source,
    binary,
    sha256,
    bytes,
    targetDirectory,
    ...details,
  });
}

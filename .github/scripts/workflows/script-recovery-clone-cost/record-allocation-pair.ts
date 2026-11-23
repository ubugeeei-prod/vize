import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile, writeJson, fileSha256, environment } from "../shared/files.ts";

import { linuxCpuAffinity } from "../shared/cpu-affinity.ts";
const root = "allocation-evidence";
function roleSource(role: string) {
  const sourceRoot =
    role === "base"
      ? join(requiredEnv("RUNNER_TEMP"), "recovery-allocation-base")
      : requiredEnv("GITHUB_WORKSPACE");
  return {
    sourceRoot,
    recoverySha256: fileSha256(
      join(sourceRoot, "crates/vize_croquis/src/script_parser/recovery.rs"),
    ),
    lockSha256: fileSha256(join(sourceRoot, "Cargo.lock")),
    carrierSha256: fileSha256(
      join(sourceRoot, "davinci/vize_l0/examples/source_recovery_clone_benchmark.rs"),
    ),
  };
}
const roles = { base: roleSource("base"), head: roleSource("head") };
assert.equal(roles.base.lockSha256, roles.head.lockSha256);
assert.equal(roles.base.carrierSha256, roles.head.carrierSha256);
const base = textFile(join(root, "base.sha")).trim();
writeJson(join(root, "identity.json"), {
  base,
  firstParent: base,
  declaredBase: requiredEnv("DECLARED_BASE_SHA"),
  head: textFile(join(root, "head.sha")).trim(),
  sourceDiff: textFile(join(root, "source-diff.txt")).trimEnd().split(/\r?\n/u).filter(Boolean),
  normalTimingQualified: false,
  roles,
});
writeJson(join(root, "resources.json"), {
  runner: environment([
    "RUNNER_NAME",
    "RUNNER_OS",
    "RUNNER_ARCH",
    "RUNNER_ENVIRONMENT",
    "CARGO_BUILD_JOBS",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
  ]),
  affinity: linuxCpuAffinity(),
  scope: "untimed allocation/source-count law only; instrumented elapsedNs is unqualified",
});

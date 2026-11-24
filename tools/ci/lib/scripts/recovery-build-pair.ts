import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile, writeJson, fileSha256 } from "../shared/files.ts";

import { cargoArtifacts } from "./cargo-artifacts.ts";
const carrierName = process.argv[2];
assert.ok(carrierName);
const root = "evidence",
  carrierFile = "davinci/vize_l0/examples/" + carrierName + ".rs";
const read = (name: string) => textFile(join(root, name)).trim();
const base = read("base.sha"),
  firstParent = read("first-parent.sha");
assert.equal(base, firstParent);
function roleBuild(role: string) {
  const sourceRoot =
    role === "base"
      ? join(requiredEnv("RUNNER_TEMP"), "recovery-base")
      : requiredEnv("GITHUB_WORKSPACE");
  const rows = cargoArtifacts(join(root, "build-" + role + ".jsonl"));
  function select(name: string, kind: string) {
    const result = rows.filter(
      (row) =>
        row.target.name === name && row.target.kind.length === 1 && row.target.kind[0] === kind,
    );
    assert.equal(result.length, 1, name);
    return result[0]!;
  }
  const croquis = select("vize_croquis", "lib"),
    carrier = select(carrierName!, "example");
  assert.equal(croquis.manifest_path, join(sourceRoot, "crates/vize_croquis/Cargo.toml"));
  assert.equal(carrier.target.src_path, join(sourceRoot, carrierFile));
  const binarySha256 = fileSha256(carrier.executable);
  assert.equal(binarySha256, fileSha256(join(root, "normal-" + role)));
  const artifacts = [croquis, carrier].map(
    ({ package_id, manifest_path, target, profile, features, fresh, executable }) => ({
      package_id,
      manifest_path,
      target,
      profile,
      features,
      fresh,
      executable,
    }),
  );
  return {
    sourceRoot,
    recoverySha256: fileSha256(
      join(sourceRoot, "crates/vize_croquis/src/script_parser/recovery.rs"),
    ),
    lockSha256: fileSha256(join(sourceRoot, "Cargo.lock")),
    carrierSha256: fileSha256(join(sourceRoot, carrierFile)),
    binarySha256,
    artifacts,
  };
}
const roles = { base: roleBuild("base"), head: roleBuild("head") };
assert.equal(roles.base.lockSha256, roles.head.lockSha256);
assert.equal(roles.base.carrierSha256, roles.head.carrierSha256);
const comparison =
  carrierName === "source_recovery_clone_benchmark"
    ? {
        trackedInputsMatch: roles.base.recoverySha256 === roles.head.recoverySha256,
        binariesMatch: roles.base.binarySha256 === roles.head.binarySha256,
      }
    : {};
writeJson(join(root, "identity.json"), {
  declaredBase: read("declared-base.sha"),
  base,
  head: read("head.sha"),
  firstParent,
  sourceDiff: read("source-diff.txt").split(/\r?\n/u).filter(Boolean),
  roles,
  ...comparison,
});

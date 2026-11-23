import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, writeJson, fileSha256, fileIdentity } from "../shared/files.ts";

import { execFileSync } from "node:child_process";
import { statSync } from "node:fs";
interface Build {
  sourceSha: string;
  sourceTree: string;
  schema: string;
  version: number;
  targetStartedAbsent: boolean;
  profile: string;
  binaryBytes: number;
  cargo: { args: string[]; version: string };
  rustc: string;
  binarySha256: string;
  differentialReceiptSha256: string;
}
const root = requiredEnv("RUNNER_TEMP"),
  custody = join(root, "helper-baseline-custody");
const sourceBuildFile = join(custody, "source-build.json"),
  build = readJson<Build>(sourceBuildFile);
const binary = join(root, "helper-image/target/ci/vize"),
  sidecar = binary + ".differential-build.json",
  linker = join(root, "wild-install/wild");
const recipe = [
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
assert.equal(build.sourceSha, "1d1bb26a7f7dcbcb9adf75075bd7ad301c0bb957");
assert.equal(build.sourceTree, "9bcbc782b84264d04190281f8acdf386c49baf64");
assert.equal(build.schema, "vize.cold-source-build");
assert.equal(build.version, 1);
assert.equal(build.targetStartedAbsent, true);
assert.equal(build.profile, "release");
assert.equal(build.binaryBytes, 56609048);
assert.deepEqual(build.cargo.args, recipe);
for (const version of [build.cargo.version, build.rustc])
  assert.ok(version.includes("\nrelease: 1.99.1\n"));
const binarySha256 = fileSha256(binary),
  sidecarSha = fileSha256(sidecar);
assert.equal(build.binarySha256, binarySha256);
assert.equal(build.differentialReceiptSha256, sidecarSha);
const receipt = readJson<{ sourceRevision: string; binarySha256: string }>(sidecar);
assert.equal(receipt.sourceRevision, build.sourceSha);
assert.equal(receipt.binarySha256, binarySha256);
const versionOutput = execFileSync(linker, ["--version"], { encoding: "utf8" }).trim();
assert.match(versionOutput, /\b0\.9\.0\b/u);
writeJson(join(custody, "regeneration.json"), {
  schema: "vize.canon-helper-source-regeneration",
  version: 1,
  sourceRevision: build.sourceSha,
  sourceTree: build.sourceTree,
  binaryBytes: statSync(binary).size,
  binarySha256,
  differentialReceiptSha256: sidecarSha,
  sourceBuildReceipt: { file: "source-build.json", sha256: fileSha256(sourceBuildFile) },
  buildRecipe: recipe,
  toolchain: {
    rust: "1.99.1",
    cargoVersion: build.cargo.version,
    rustcVersion: build.rustc,
    linker: {
      name: "wild",
      version: "0.9.0",
      actionRevision: "0bbbfa5df4380cab8e63cb8505a1ce65e1d10203",
      ...fileIdentity(linker),
      versionOutput,
    },
  },
  historicalBinarySha256: "cdf44149ddfe160339887a256c1bd0253fb34f1f7c774759e6c5382b42ed0cb9",
  historicalArchiveDigests: {
    sourceImage: "85be25beb129752cebab4c5e96bba80505e82b14f43115b526beee8dc6dd853b",
    metadata: "c6010fed95cfd24975b941b54011715a887d731678faf44acfde54e1702a1107",
  },
});

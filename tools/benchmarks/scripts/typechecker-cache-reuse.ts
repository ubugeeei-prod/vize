/** Reuse one immutable normal image; never relabel its source revision as the current checkout. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: { image: { type: "string" }, build: { type: "string" }, out: { type: "string" } },
});
assert.ok(values.image && values.build && values.out);
const image = fs.realpathSync(values.image),
  buildPath = fs.realpathSync(values.build),
  out = path.resolve(values.out);
assert.ok(!fs.existsSync(out));
fs.mkdirSync(out, { recursive: true });
const sha = (p: string) => createHash("sha256").update(fs.readFileSync(p)).digest("hex");
const build = JSON.parse(fs.readFileSync(buildPath, "utf8"));
const sidecar = image + ".differential-build.json";
const canonical = JSON.parse(fs.readFileSync(sidecar, "utf8"));
const source = "fb550c424e4256860647dbf96ce32c31cea3774b";
const expected = "cdf44149ddfe160339887a256c1bd0253fb34f1f7c774759e6c5382b42ed0cb9";
assert.equal(sha(image), expected);
assert.equal(fs.statSync(image).size, 55_080_984);
assert.equal(build.schema, "vize.cold-source-build");
assert.equal(build.version, 1);
assert.equal(build.sourceSha, source);
assert.equal(build.binarySha256, expected);
assert.equal(build.sourceTree, "066589f286ccaf1e271dc9b2e3c87c81d6db9013");
assert.equal(build.targetStartedAbsent, true);
assert.equal(build.cargoArtifact.fresh, false);
assert.deepEqual(build.cargoArtifact.features, ["default", "glyph", "maestro"]);
assert.deepEqual(build.cargoArtifact.profile, {
  opt_level: "3",
  debuginfo: 0,
  debug_assertions: false,
  overflow_checks: false,
  test: false,
});
assert.equal(build.profile, "release");
assert.equal(canonical.schema, "vize.differential.build");
assert.equal(canonical.version, 1);
assert.equal(canonical.recipe, "cargo build --release -p vize");
assert.equal(canonical.sourceRevision, source);
assert.equal(canonical.binarySha256, expected);
assert.equal(canonical.cliVersion, "vize 0.474.0-dev");
assert.equal(sha(sidecar), build.differentialReceiptSha256);
const probe = spawnSync(image, ["--version"], { encoding: "utf8", timeout: 30_000 });
assert.equal(probe.error, undefined);
assert.equal(probe.status, 0);
assert.equal(probe.stdout.trim(), canonical.cliVersion);
fs.copyFileSync(buildPath, path.join(out, "source-build.json"));
fs.copyFileSync(sidecar, path.join(out, "differential-build.json"));
fs.writeFileSync(
  path.join(out, "reuse.json"),
  JSON.stringify(
    {
      schema: "vize.normal-checkpoint-reuse",
      version: 1,
      passed: true,
      sourceSha: source,
      binarySha256: expected,
      cliVersion: canonical.cliVersion,
      sourceRun: 40570360508,
      imageArtifact: 12765860439,
      metadataArtifact: 12764104945,
      buildReceiptSha256: sha(buildPath),
      directRootAttested: false,
      scope: "normal immutable source checkpoint; no observer feature; not the current checkout",
    },
    null,
    2,
  ) + "\n",
);
console.log(
  JSON.stringify({
    sourceSha: source,
    binarySha256: expected,
    cliVersion: canonical.cliVersion,
    passed: true,
  }),
);

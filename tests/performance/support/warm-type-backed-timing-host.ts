import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type Side = "before" | "after";
type BodyDigest = (side: Side, file: string) => string | null;
type Manifest = {
  version: number;
  originalSource: string;
  transformedSource: string;
  files: Array<[string, string | null, string | null]>;
  preservedBodies: Array<[string, string]>;
};

const manifest = JSON.parse(
  fs.readFileSync(new URL("./warm-type-backed-timing-host-manifest.json", import.meta.url), "utf8"),
) as Manifest;
const manifestSHA256 = createHash("sha256").update(JSON.stringify(manifest)).digest("hex");
assert.equal(manifestSHA256, "534d1d12244a4bff9c6ed4fff487e1f601ef52ea975c4dab080030266f5a4062");
assert.equal(manifest.version, 1);
assert.equal(manifest.originalSource, "b41811e3b33676f68a0843a693c9ed4898f2e86a");
assert.match(manifest.transformedSource, /^[a-f0-9]{40}$/u);

const oldLaw = "davinci/vize_l0/tests/pass_observer_timing.rs";
const expectedFiles = [
  "crates/vize/src/commands/davinci_ice.rs",
  "crates/vize_carton/src/lib.rs",
  "crates/vize_carton/src/timing_observer.rs",
  "crates/vize_carton/tests/pass_observer_timing.rs",
  "crates/vize_curator/src/inspector/stages/profile.rs",
  "davinci/vize_l0/src/pass.rs",
  "davinci/vize_l0/src/pass/observer.rs",
  "davinci/vize_l0/src/pass/observer/timing.rs",
  "davinci/vize_l0/tests/dump_collector.rs",
  oldLaw,
  "davinci/vize_l0/tests/remark_zero_cost.rs",
].toSorted();
assert.equal(expectedFiles.length, 11);
assert.deepEqual(
  manifest.files.map(([file]) => file),
  expectedFiles,
);
for (const [, before, after] of manifest.files)
  for (const digest of [before, after])
    if (digest !== null) assert.match(digest, /^[a-f0-9]{64}$/u);
assert.deepEqual(manifest.preservedBodies, [
  [
    "davinci/vize_l0/src/pass/observer/timing/walk.rs",
    "3887b2faed4ef43a4a8e34fe64f2fe74eda1f12642f65561fd8979be3d098a4e",
  ],
  [
    "davinci/vize_l0/tests/walk_timing.rs",
    "1ea1009073ec3573f1c9e43ef16d77fe60e85b762bc3f298ea2806cb2161540f",
  ],
]);

/** Admit only the complete reviewed host extraction and its unchanged portable laws. */
export function qualifyTimingHostMove(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: BodyDigest,
) {
  if (new Set(production).size !== production.length) return null;
  const files = new Set(expectedFiles);
  if (production.every((file) => alreadyQualified.has(file))) return null;
  if (production.some((file) => !alreadyQualified.has(file) && !files.has(file))) return null;
  // Rename detection may omit the deleted law; every retained and new owner must appear.
  const footprint = production.filter((file) => files.has(file) && file !== oldLaw).toSorted();
  if (JSON.stringify(footprint) !== JSON.stringify(expectedFiles.filter((file) => file !== oldLaw)))
    return null;
  for (const [file, before, after] of manifest.files)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  for (const [file, original] of manifest.preservedBodies)
    if (digest("before", file) !== original || digest("after", file) !== original) return null;
  return {
    authority: "exact-reviewed-timing-observer-host-move",
    manifestSHA256,
    originalSource: manifest.originalSource,
    transformedSource: manifest.transformedSource,
    completeBodyPairs: manifest.files.length,
    preservedBodyPairs: manifest.preservedBodies.length,
    files: expectedFiles,
  };
}

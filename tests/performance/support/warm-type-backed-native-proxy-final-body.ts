import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type Digest = (side: "before" | "after", file: string) => string | null;
type Body = [string, string | null, string];
const read = (name: string) => JSON.parse(fs.readFileSync(new URL(name, import.meta.url), "utf8"));
const hash = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
const bytes = fs.readFileSync(
  new URL("./warm-type-backed-native-proxy-final-body-manifest.json", import.meta.url),
);
export const nativeProxyFinalManifestBodySHA256 = createHash("sha256").update(bytes).digest("hex");
assert.equal(
  nativeProxyFinalManifestBodySHA256,
  "6c2f6d6a1431e562e608762bb017ef77a5dc667a24456d24a74fe5966c870f24",
);
export const nativeProxyFinalManifest = JSON.parse(bytes.toString("utf8")) as {
  version: number;
  beforeSource: string;
  previousSource: string;
  file: string;
  beforeSHA256: string | null;
  previousAfterSHA256: string;
  afterSHA256: string;
};
const manifest = nativeProxyFinalManifest;
const manifestSHA256 = hash(manifest);
assert.equal(manifestSHA256, "50dc0114cbd42ac8595093f8eebc8c2054bd67e09a45896edc6bd921f2004da0");
assert.equal(manifest.version, 1);
assert.equal(manifest.file, "tests/performance/support/warm-type-backed-native-terminal-final.ts");
if (manifest.beforeSHA256 !== null) assert.match(manifest.beforeSHA256, /^[a-f0-9]{64}$/u);
for (const digest of [manifest.previousAfterSHA256, manifest.afterSHA256])
  assert.match(digest, /^[a-f0-9]{64}$/u);

// Independent data custody: this module never imports a source qualifier.
const fixture = read("./warm-type-backed-native-proxy-manifest.json") as {
  reviewedSource: string;
  reviewedSourceTree: string;
  files: Body[];
};
const correction = read("./warm-type-backed-native-proxy-correction.json") as {
  file: string;
  before: string;
  after: string;
  reviewedSource: string;
  reviewedSourceTree: string;
};
assert.equal(hash(fixture), "d90b51370f0eaaa6e7459477d05dd0d225cd88e91fed9bbeb11e04083f0e2680");
assert.equal(hash(correction), "a5ba4433860ea0c3c2cd53c4f5eb00cc5d4c079ba62b221c33a5581620d0b42b");
assert.equal(fixture.files.length, 7);
assert.equal(new Set(fixture.files.map(([file]) => file)).size, 7);
assert.equal(fixture.files.find(([file]) => file === correction.file)?.[2], correction.before);
export const nativeProxyFinalFixtureVariants = [
  {
    reviewedSource: fixture.reviewedSource,
    reviewedSourceTree: fixture.reviewedSourceTree,
    files: fixture.files,
  },
  {
    reviewedSource: correction.reviewedSource,
    reviewedSourceTree: correction.reviewedSourceTree,
    files: fixture.files.map(
      ([file, before, after]) =>
        [file, before, file === correction.file ? correction.after : after] as Body,
    ),
  },
];

/** Final custody checks the whole stored packet, without accepting missing observations. */
export function assertNativeProxyFinalFixtureReceipt(
  observed: unknown,
  stored: unknown,
): asserts observed {
  assert.ok(
    observed,
    "complete reviewed native proxy fixture custody is required at final comparison",
  );
  assert.deepEqual(observed, stored ?? null);
}

/** One exact final caller successor, only alongside a complete reviewed fixture variant. */
export function qualifyNativeProxyFinalBody(production: string[], digest: Digest) {
  if (new Set(production).size !== production.length) return null;
  if (
    digest("before", manifest.file) !== manifest.beforeSHA256 ||
    digest("after", manifest.file) !== manifest.afterSHA256
  )
    return null;
  const files = fixture.files.map(([file]) => file).toSorted();
  const owned = new Set(files);
  if (
    JSON.stringify(production.filter((file) => owned.has(file)).toSorted()) !==
    JSON.stringify(files)
  )
    return null;
  const selected = nativeProxyFinalFixtureVariants.find(({ files }) =>
    files.every(
      ([file, before, after]) =>
        digest("before", file) === before && digest("after", file) === after,
    ),
  );
  if (!selected) return null;
  return {
    authority: "exact-reviewed-native-proxy-final-caller-successor",
    manifestSHA256,
    file: manifest.file,
    beforeSHA256: manifest.beforeSHA256,
    previousAfterSHA256: manifest.previousAfterSHA256,
    afterSHA256: manifest.afterSHA256,
    completeCallerBodyPairs: 1,
    originalRequiredHarnessBodyPairs: 2,
    fixtureReviewedSource: selected.reviewedSource,
    fixtureReviewedSourceTree: selected.reviewedSourceTree,
    completeFixtureBodyPairs: 7,
    files,
    runtimeCredit: false,
    performanceCredit: false,
  };
}

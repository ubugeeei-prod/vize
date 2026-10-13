import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { nativeLexicalCompositionBodyPairs } from "./warm-type-backed-native-lexical-composition-host.ts";

type Digest = (side: "before" | "after", file: string) => string | null;
export const nativeProxyManifest = JSON.parse(
  fs.readFileSync(
    new URL("./warm-type-backed-native-proxy-manifest.json", import.meta.url),
    "utf8",
  ),
) as {
  version: number;
  beforeSource: string;
  reviewedSource: string;
  reviewedSourceTree: string;
  workflowBeforeSHA256: string;
  files: Array<[string, string | null, string]>;
};
const manifestSHA256 = createHash("sha256")
  .update(JSON.stringify(nativeProxyManifest))
  .digest("hex");
assert.equal(manifestSHA256, "d90b51370f0eaaa6e7459477d05dd0d225cd88e91fed9bbeb11e04083f0e2680");
assert.equal(nativeProxyManifest.version, 1);
assert.equal(nativeProxyManifest.files.length, 7);
assert.equal(new Set(nativeProxyManifest.files.map(([file]) => file)).size, 7);
for (const [file, before, after] of nativeProxyManifest.files) {
  assert.match(
    file,
    /^crates\/vize_maestro\/(?:src\/ide\/rename\/corsa_session_tests\/harness\/protocol|tests\/fixtures\/native-proxy-)/u,
  );
  if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
  assert.match(after, /^[a-f0-9]{64}$/u);
}

const correction = JSON.parse(
  fs.readFileSync(
    new URL("./warm-type-backed-native-proxy-correction.json", import.meta.url),
    "utf8",
  ),
) as {
  version: number;
  beforeSource: string;
  reviewedSource: string;
  reviewedSourceTree: string;
  file: string;
  before: string;
  after: string;
};
assert.equal(
  createHash("sha256").update(JSON.stringify(correction)).digest("hex"),
  "a5ba4433860ea0c3c2cd53c4f5eb00cc5d4c079ba62b221c33a5581620d0b42b",
);
assert.equal(correction.version, 1);
assert.equal(
  correction.file,
  "crates/vize_maestro/src/ide/rename/corsa_session_tests/harness/protocol/lifecycle_tests.rs",
);
assert.equal(
  nativeProxyManifest.files.find(([file]) => file === correction.file)?.[2],
  correction.before,
);
assert.match(correction.after, /^[a-f0-9]{64}$/u);
export const nativeProxyBodyPairs = nativeProxyManifest.files.map(
  ([file, before, after]) =>
    [file, before, file === correction.file ? correction.after : after] as const,
);

/** Seven whole fixture bodies only; the original complete SDK source guard remains mandatory. */
export function qualifyNativeProxyHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: Digest,
) {
  const files = nativeProxyManifest.files.map(([file]) => file).toSorted();
  const owned = new Set(files);
  const sdk = new Set(nativeLexicalCompositionBodyPairs.map(([file]) => file));
  if (new Set(production).size !== production.length) return null;
  if (production.some((file) => !owned.has(file) && !sdk.has(file) && !alreadyQualified.has(file)))
    return null;
  if (
    JSON.stringify(production.filter((file) => owned.has(file)).toSorted()) !==
    JSON.stringify(files)
  )
    return null;
  const corrected = digest("after", correction.file) === correction.after;
  for (const [file, before, after] of corrected ? nativeProxyBodyPairs : nativeProxyManifest.files)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-reviewed-authored-native-proxy-fixtures",
    manifestSHA256,
    beforeSource: nativeProxyManifest.beforeSource,
    reviewedSource: corrected ? correction.reviewedSource : nativeProxyManifest.reviewedSource,
    reviewedSourceTree: corrected
      ? correction.reviewedSourceTree
      : nativeProxyManifest.reviewedSourceTree,
    wholeFixtureBodyPairs: 7,
    files,
    runtimeCredit: false,
    performanceCredit: false,
  };
}

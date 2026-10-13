import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "./support/native-sdk-historical-source.mjs";
import { test } from "node:test";
import { reviewedHostPaths } from "../performance/support/warm-type-backed-reviewed-hosts.ts";
import { nativeOwnedImageManifest as sdk } from "../performance/support/warm-type-backed-native-owned-image-host.ts";
import { nativeTerminalManifest as original } from "../performance/support/warm-type-backed-native-terminal-host.ts";
import {
  nativeLexicalCompositionBodyPairs,
  nativeLexicalCompositionManifest as composition,
  qualifyNativeTerminalHost,
} from "../performance/support/warm-type-backed-native-lexical-composition-host.ts";
import { qualifyNativeProxyHost } from "../performance/support/warm-type-backed-native-proxy-host.ts";
import {
  nativeProxyFinalFixtureVariants,
  nativeProxyFinalManifest as final,
  qualifyNativeProxyFinalBody,
  assertNativeProxyFinalFixtureReceipt,
} from "../performance/support/warm-type-backed-native-proxy-final-body.ts";

type Side = "before" | "after";
type Entry = { before: string | null; after: string };
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const entries = new Map<string, Entry>();
const put = (file: string, before: string | null, after: string) => {
  const previous = entries.get(file);
  if (previous) assert.deepEqual(previous, { before, after });
  entries.set(file, { before, after });
};
const lexical = JSON.parse(
  fs.readFileSync(
    new URL(
      "../performance/support/warm-type-backed-lexical-dollar-manifest.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  files: Array<[string, string | null, string]>;
  corpusBodies: Array<[string, string | null, string]>;
  preservedBodies: Array<[string, string]>;
};
for (const [file, before, after] of [
  ...nativeLexicalCompositionBodyPairs,
  ...sdk.corpusBodies,
  ...composition.historicalReplayBodies,
  ...nativeProxyFinalFixtureVariants[1].files,
])
  put(file, before, after);
for (const [file, , accepted] of [...lexical.files, ...lexical.corpusBodies])
  put(
    file,
    accepted,
    composition.sourceSuccessors.find(([name]) => name === file)?.[3] ?? accepted,
  );
for (const [file, body] of lexical.preservedBodies)
  put(
    file,
    body,
    composition.incomingPreservedSuccessors.find(([name]) => name === file)?.[2] ?? body,
  );
const harness = original.qualifiedHarnessBodies.map(
  ([file, before, after]) =>
    [
      file,
      before,
      file === final.file
        ? final.afterSHA256
        : (composition.qualifiedHarnessSuccessors.find(([name]) => name === file)?.[2] ??
          sdk.qualifiedHarnessBodies.find(([name]) => name === file)?.[2] ??
          after),
    ] as const,
);
for (const [file, before, after] of harness) put(file, before, after);
for (const [file, body] of original.preservedBodies)
  put(file, body, harness.find(([name]) => name === file)?.[2] ?? body);
const helper = "tests/performance/support/warm-type-backed-native-proxy-final-body.ts";
const metadata = "tests/performance/support/warm-type-backed-native-proxy-final-body-manifest.json";
for (const file of [helper, metadata])
  put(file, null, sha(fs.readFileSync(new URL("../../" + file, import.meta.url))));
const digest = (side: Side, file: string) => entries.get(file)?.[side] ?? null;
const production = [
  ...nativeLexicalCompositionBodyPairs.map(([file]) => file),
  ...nativeProxyFinalFixtureVariants[1].files.map(([file]) => file),
];
const allowed = new Set(reviewedHostPaths);
const fixture = () => {
  const receipt = qualifyNativeProxyHost(production, allowed, digest);
  assert.ok(receipt);
  return receipt;
};
const sdkQualification = (read = digest, paths = production) =>
  qualifyNativeTerminalHost(paths, new Set([...allowed, ...fixture().files]), read);

test("both complete reviewed fixture variants qualify one actual whole final caller", () => {
  for (const variant of nativeProxyFinalFixtureVariants) {
    const bodies = new Map(variant.files.map(([file, before, after]) => [file, { before, after }]));
    const read = (side: Side, file: string) => bodies.get(file)?.[side] ?? digest(side, file);
    const receipt = qualifyNativeProxyFinalBody(production, read);
    assert.ok(receipt);
    assert.equal(receipt.fixtureReviewedSource, variant.reviewedSource);
    assert.equal(receipt.fixtureReviewedSourceTree, variant.reviewedSourceTree);
    assert.equal(receipt.completeCallerBodyPairs, 1);
    assert.equal(receipt.originalRequiredHarnessBodyPairs, 2);
    assert.equal(receipt.completeFixtureBodyPairs, 7);
    assert.equal(receipt.runtimeCredit, false);
    assert.equal(receipt.performanceCredit, false);
  }
});

test("source admission retains all original SDK and both caller authorities", () => {
  assert.equal(qualifyNativeTerminalHost(production, allowed, digest), null);
  const native = sdkQualification();
  assert.ok(native);
  assert.equal(native.completeBodyPairs, 121);
  assert.equal(native.wholeSourceBodyPairs, 122);
  assert.equal(native.preservedBodyPairs, 23);
  assert.equal(native.qualifiedHarnessBodyPairs, 2);
  assert.ok("nativeProxyFinalBody" in native);
  assert.deepEqual(native.nativeProxyFinalBody, qualifyNativeProxyFinalBody(production, digest));
});

test("missing partial duplicate foreign and unknown fixture sets refuse", () => {
  for (const [file] of nativeProxyFinalFixtureVariants[1].files) {
    assert.equal(
      qualifyNativeProxyFinalBody(
        production.filter((name) => name !== file),
        digest,
      ),
      null,
    );
    assert.equal(qualifyNativeProxyFinalBody([...production, file], digest), null);
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyNativeProxyFinalBody(production, (s, name) =>
          s === side && name === file ? "f".repeat(64) : digest(s, name),
        ),
        null,
      );
  }
  assert.equal(qualifyNativeProxyFinalBody([], digest), null);
  assert.equal(
    sdkQualification(digest, [...production, "crates/vize_maestro/src/foreign.rs"]),
    null,
  );
});

test("helper metadata and one caller body require exact after bytes and genuine before absences", () => {
  for (const file of [helper, metadata, final.file])
    for (const side of ["before", "after"] as const)
      for (const bad of [null, "f".repeat(64)]) {
        if (bad === digest(side, file)) continue;
        assert.equal(
          sdkQualification((s, name) => (s === side && name === file ? bad : digest(s, name))),
          null,
          `${side}:${file}`,
        );
      }
});

test("both original caller bodies and foreign SDK sources remain mandatory", () => {
  for (const file of [
    ...harness.map(([file]) => file),
    ...nativeLexicalCompositionBodyPairs.map(([file]) => file),
  ])
    for (const side of ["before", "after"] as const)
      assert.equal(
        sdkQualification((s, name) =>
          s === side && name === file ? "f".repeat(64) : digest(s, name),
        ),
        null,
      );
});

test("final custody refuses missing extra foreign and cross-variant stored packets", () => {
  const observed = fixture();
  assert.doesNotThrow(() =>
    assertNativeProxyFinalFixtureReceipt(observed, structuredClone(observed)),
  );
  for (const bad of [
    null,
    undefined,
    {},
    { ...observed, extra: 1 },
    { ...observed, files: observed.files.slice(1) },
    { ...observed, reviewedSource: nativeProxyFinalFixtureVariants[0].reviewedSource },
    { ...observed, reviewedSourceTree: "f".repeat(40) },
  ])
    assert.throws(() => assertNativeProxyFinalFixtureReceipt(observed, bad));
  assert.throws(() => assertNativeProxyFinalFixtureReceipt(null, null));
  assert.throws(() => assertNativeProxyFinalFixtureReceipt(null, observed));
});

test("finite final source edits restore the complete original caller and keep original request body", () => {
  const body = fs.readFileSync(
    new URL("../performance/support/warm-type-backed-native-terminal-final.ts", import.meta.url),
    "utf8",
  );
  const restored = body
    .replace(
      'import { qualifyNativeProxyHost } from "./warm-type-backed-native-proxy-host.ts";\n',
      "",
    )
    .replace(
      'import { assertNativeProxyFinalFixtureReceipt } from "./warm-type-backed-native-proxy-final-body.ts";\n',
      "",
    )
    .replace("  nativeProxyHost?: ReturnType<typeof qualifyNativeProxyHost>;\n", "")
    .replace(
      `  const allowed = new Set(reviewedHostPaths);
  const fixture = qualifyNativeProxyHost(binding.production, allowed, (side, file) =>
    gitBodyDigest(driverRoot, side === "before" ? binding.baseline : binding.head, file),
  );
  assertNativeProxyFinalFixtureReceipt(fixture, binding.nativeProxyHost);
  for (const file of fixture.files) allowed.add(file);
`,
      "",
    )
    .replace(
      `  const qualified = qualifyNativeTerminalHost(binding.production, allowed, (side, file) =>
    gitBodyDigest(driverRoot, side === "before" ? binding.baseline : binding.head, file),
  );`,
      `  const qualified = qualifyNativeTerminalHost(
    binding.production,
    new Set(reviewedHostPaths),
    (side, file) =>
      gitBodyDigest(driverRoot, side === "before" ? binding.baseline : binding.head, file),
  );`,
    );
  assert.equal(sha(Buffer.from(restored)), final.previousAfterSHA256);
  const request = original.qualifiedHarnessBodies.find(
    ([file]) => file === "tests/performance/support/warm-type-backed-requests.ts",
  );
  assert.ok(request);
  assert.equal(sha(fs.readFileSync(new URL("../../" + request[0], import.meta.url))), request[2]);
});

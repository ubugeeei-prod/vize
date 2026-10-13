import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "./support/native-sdk-historical-source.mjs";
import { test } from "node:test";
import {
  assertNativeTerminalLocks,
  nativeTerminalManifest as original,
} from "../performance/support/warm-type-backed-native-terminal-host.ts";
import {
  nativeOwnedImageManifest as sdk,
  qualifyNativeTerminalHost as qualifySdk,
} from "../performance/support/warm-type-backed-native-owned-image-host.ts";
import {
  nativeLexicalCompositionBodyPairs,
  nativeLexicalCompositionManifest as composition,
  qualifyNativeTerminalHost,
} from "../performance/support/warm-type-backed-native-lexical-composition-host.ts";
import { qualifyLexicalDollarHost } from "../performance/support/warm-type-backed-lexical-dollar-host.ts";
import { validateLexicalSourceBodies } from "../../tools/support/compat/github/native-lexical-composition-custody.mjs";
import { historicalCompositionBody } from "./support/native-sdk-historical-source.mjs";

type Side = "before" | "after";
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
const entries = new Map<string, { before: string | null; after: string }>();
const put = (file: string, before: string | null, after: string) => {
  const previous = entries.get(file);
  if (previous) assert.deepEqual(previous, { before, after }, file);
  entries.set(file, { before, after });
};
for (const [file, before, after] of [
  ...nativeLexicalCompositionBodyPairs,
  ...sdk.corpusBodies,
  ...composition.historicalReplayBodies,
])
  put(file, before, after);
for (const [file, , accepted] of [...lexical.files, ...lexical.corpusBodies]) {
  const successor = composition.sourceSuccessors.find(([name]) => name === file);
  put(file, accepted, successor?.[3] ?? accepted);
}
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
      composition.qualifiedHarnessSuccessors.find(([name]) => name === file)?.[2] ??
        sdk.qualifiedHarnessBodies.find(([name]) => name === file)?.[2] ??
        after,
    ] as const,
);
for (const [file, before, after] of harness) put(file, before, after);
for (const [file, body] of original.preservedBodies)
  put(file, body, harness.find(([name]) => name === file)?.[2] ?? body);
const digest = (side: Side, file: string) => entries.get(file)?.[side] ?? null;
const production = nativeLexicalCompositionBodyPairs.map(([file]) => file);
const hash = (file: string) =>
  createHash("sha256")
    .update(fs.readFileSync(new URL("../../" + file, import.meta.url)))
    .digest("hex");
const locks = (side: Side) =>
  Object.fromEntries(
    ["Cargo.toml", "Cargo.lock", "package.json", "pnpm-lock.yaml", "rust-toolchain.toml"].map(
      (file) => [file, digest(side, file)!],
    ),
  );

const retainedFinalCapsule = JSON.parse(
  fs.readFileSync(new URL("./support/native-proxy-original-final.json", import.meta.url), "utf8"),
) as Record<string, unknown>;
const retainedOriginalFinalDigest = (capsule: unknown) => {
  assert.ok(capsule && typeof capsule === "object" && !Array.isArray(capsule));
  const { rawBase64, ...identity } = capsule as Record<string, unknown>;
  assert.deepEqual(identity, {
    version: 1,
    source: "e17c27d04521ca1409ffb65b7c700d417fc66733",
    file: "tests/performance/support/warm-type-backed-native-terminal-final.ts",
    mode: "100644",
    blob: "20ad1caaaaf3cdea63ed741a81ad0dfdb8e85345",
    bytes: 1144,
    sha256: "e5cf6338b4edbc08641aee70888b79fe0d69b27118571fcb86cd5041479f503a",
  });
  assert.ok(typeof rawBase64 === "string");
  const retained = Buffer.from(rawBase64, "base64");
  assert.equal(retained.toString("base64"), rawBase64);
  assert.equal(retained.length, identity.bytes);
  assert.equal(
    createHash("sha1")
      .update(Buffer.concat([Buffer.from(`blob ${retained.length}\0`), retained]))
      .digest("hex"),
    identity.blob,
  );
  const retainedDigest = createHash("sha256").update(retained).digest("hex");
  assert.equal(retainedDigest, identity.sha256);
  return retainedDigest;
};

test("retained original guard authenticates complete composition and both immutable authorities", () => {
  const final = "tests/performance/support/warm-type-backed-native-terminal-final.ts";
  const retainedDigest = retainedOriginalFinalDigest(retainedFinalCapsule);
  const actual = (side: Side, file: string) =>
    side === "before" ? digest(side, file) : file === final ? retainedDigest : hash(file);
  assert.equal(qualifySdk(production, new Set(), actual), null);
  assert.equal(
    qualifyLexicalDollarHost(
      lexical.files.map(([file]) => file),
      new Set(),
      actual,
    ),
    null,
  );
  const result = qualifyNativeTerminalHost(production, new Set(), actual);
  assert.ok(result && "incomingLexical" in result);
  assert.equal(result.completeBodyPairs, 121);
  assert.equal(result.wholeSourceBodyPairs, 122);
  assert.equal(result.preservedBodyPairs, 23);
  assert.equal(result.qualifiedHarnessBodyPairs, 2);
  assert.deepEqual(result.incomingLexical, {
    manifestSHA256: composition.lexicalManifestSHA256,
    completeBodyPairs: 9,
    completeCorpusBodyPairs: 21,
    preservedBodyPairs: 7,
  });
  assert.equal(result.historicalReplayBodyPairs, 7);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
  assertNativeTerminalLocks(locks("before"), locks("after"), result);
});

test("whole original32 legacy cases and every existing manifest field survive the one existing SDK supplement", () => {
  const file = "tests/_fixtures/differential/typechecker/manifest.json";
  const before = JSON.parse(historicalCompositionBody("lexical", file).toString());
  const after = JSON.parse(fs.readFileSync(new URL("../../" + file, import.meta.url), "utf8"));
  assert.equal(before.cases.length, 32);
  assert.equal(after.supplementalCases.length, 1);
  assert.equal(after.supplementalCases[0].contract, "source-canon-semantic-completion-v1");
  const { supplementalCases: _supplement, ...remaining } = after;
  assert.deepEqual(remaining, before);
});

test("each whole source, corpus, runtime, guard and historical replay body refuses absence or mutation on either side", () => {
  for (const [file] of entries)
    for (const side of ["before", "after"] as const)
      for (const change of [null, "f".repeat(64)]) {
        if (change === digest(side, file)) continue;
        assert.equal(
          qualifyNativeTerminalHost(production, new Set(), (s, name) =>
            s === side && name === file ? change : digest(s, name),
          ),
          null,
          `${side}:${file}`,
        );
      }
});

test("missing, duplicate, unknown and entirely prequalified production cannot enroll", () => {
  for (const file of production) {
    assert.equal(
      qualifyNativeTerminalHost(
        production.filter((name) => name !== file),
        new Set(),
        digest,
      ),
      null,
    );
    assert.equal(qualifyNativeTerminalHost([...production, file], new Set(), digest), null);
  }
  assert.equal(
    qualifyNativeTerminalHost(
      [...production, "crates/vize_canon/src/foreign.rs"],
      new Set(),
      digest,
    ),
    null,
  );
  assert.equal(qualifyNativeTerminalHost(production, new Set(production), digest), null);
});

test("mixing either old after body into only one composed producer is refused", () => {
  for (const [file, sdkAfter, acceptedAfter] of composition.sourceSuccessors)
    for (const oldBody of [sdkAfter, acceptedAfter])
      assert.equal(
        qualifyNativeTerminalHost(production, new Set(), (side, name) =>
          side === "after" && name === file ? oldBody : digest(side, name),
        ),
        null,
      );
});

test("complete source custody does not exempt changed provider or Cargo locks", () => {
  const result = qualifyNativeTerminalHost(production, new Set(), digest);
  assert.ok(result);
  for (const side of ["before", "after"] as const)
    for (const file of Object.keys(locks(side))) {
      const before = locks("before"),
        after = locks("after");
      (side === "before" ? before : after)[file] = "f".repeat(64);
      assert.throws(() => assertNativeTerminalLocks(before, after, result));
    }
});

test("native lexical source custody accepts only original complete lexical or whole composed SDK source", () => {
  const copied = [...entries].map(([file, body]) => ({ file, sha256: body.after }));
  for (const file of [
    "warm-type-backed-native-lexical-composition-manifest.json",
    "warm-type-backed-native-owned-image-manifest.json",
    "warm-type-backed-native-terminal-manifest.json",
    "warm-type-backed-native-terminal-members-1.json",
    "warm-type-backed-native-terminal-members-2.json",
  ])
    copied.push({
      file: "tests/performance/support/" + file,
      sha256: hash("tests/performance/support/" + file),
    });
  assert.ok(validateLexicalSourceBodies(lexical, copied));
  const old = copied.map((entry) => ({
    ...entry,
    sha256: lexical.files.find(([file]) => file === entry.file)?.[2] ?? entry.sha256,
  }));
  assert.equal(validateLexicalSourceBodies(lexical, old), null);
  for (const [file] of entries) {
    const changed = copied.map((entry) =>
      entry.file === file ? { ...entry, sha256: "f".repeat(64) } : entry,
    );
    assert.throws(() => validateLexicalSourceBodies(lexical, changed), undefined, file);
  }
});

test("self-contained original final capsule refuses missing foreign and changed source bytes without Git", () => {
  assert.equal(
    retainedOriginalFinalDigest(retainedFinalCapsule),
    "e5cf6338b4edbc08641aee70888b79fe0d69b27118571fcb86cd5041479f503a",
  );
  for (const field of Object.keys(retainedFinalCapsule)) {
    const missing = { ...retainedFinalCapsule };
    delete missing[field];
    assert.throws(() => retainedOriginalFinalDigest(missing), undefined, field);
    assert.throws(
      () => retainedOriginalFinalDigest({ ...retainedFinalCapsule, [field]: "foreign" }),
      undefined,
      field,
    );
  }
  assert.throws(() => retainedOriginalFinalDigest({ ...retainedFinalCapsule, unknown: true }));
  const bytes = Buffer.from(retainedFinalCapsule.rawBase64 as string, "base64");
  bytes[0] ^= 1;
  assert.throws(() =>
    retainedOriginalFinalDigest({ ...retainedFinalCapsule, rawBase64: bytes.toString("base64") }),
  );
  assert.throws(() =>
    retainedOriginalFinalDigest({
      ...retainedFinalCapsule,
      rawBase64: String(retainedFinalCapsule.rawBase64) + "\n",
    }),
  );
});

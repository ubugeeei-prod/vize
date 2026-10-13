// Byte-backed source qualification only: no compiler, native process or performance credit.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { runOwnedCargoOriginalRefusal } from "./support/owned-corsa-cargo-original-refusal.mjs";
import {
  lexicalFinalSourcePaths,
  lexicalSourcePaths as originalSourcePaths,
  validateLexicalFinalSource,
  validateLexicalSourceBodies,
} from "../../tools/support/compat/github/native-lexical-composition-custody.mjs";
import {
  sourcePaths,
  helperPaths,
} from "../../tools/support/compat/github/native-lexical-dollar-metadata.mjs";
import { projectOwnedCorsaLexicalCopied } from "../../tools/support/compat/github/owned-corsa-lexical-source.mjs";
import { ownedCorsaSource } from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { nativeLexicalCompositionBodyPairs } from "../performance/support/warm-type-backed-native-lexical-composition-host.ts";
import { nativeProxyManifest } from "../performance/support/warm-type-backed-native-proxy-host.ts";
import { historicalCompositionBody } from "./support/native-sdk-historical-source.mjs";

const root = new URL("../../", import.meta.url);
const rootPath = fileURLToPath(root);
const read = (file) => fs.readFileSync(new URL(file, root));
const sha = (body) => createHash("sha256").update(body).digest("hex");
const manifest = JSON.parse(
  read("tests/performance/support/warm-type-backed-lexical-dollar-manifest.json"),
);
const caller = "tests/performance/support/warm-type-backed-native-terminal-final.ts";
const oldSHA = "e5cf6338b4edbc08641aee70888b79fe0d69b27118571fcb86cd5041479f503a";
const currentSHA = "f5a60202ec1ff1b149cbb320ac3c8f7382ff85a2c9a1c8cb5ebafed863bdccbc";
const previous = read("tests/_fixtures/ci/native-lexical-final-source-before.mjs.txt").toString();
assert.equal(
  sha(Buffer.from(previous)),
  "420060a317e82edd598b67979787307057a6623f9b07b302bde64019bff1ba59",
);
const rootClause = 'const root = new URL("../../../../", import.meta.url);';
const transported = `const root = new URL(${JSON.stringify(root.href)});`;
assert.equal(previous.split(rootClause).length, 2);
const original = previous.replace(rootClause, transported);
assert.equal(original.replace(transported, rootClause), previous);
const old = await import(`data:text/javascript;base64,${Buffer.from(original).toString("base64")}`);
const currentHead = execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
  cwd: rootPath,
})
  .toString()
  .trim();
const copied = sourcePaths(manifest).map((file) => {
  const body = read(file);
  assert.deepEqual(
    body,
    execFileSync("git", ["--no-replace-objects", "show", `${currentHead}:${file}`], {
      cwd: rootPath,
    }),
  );
  const gitEntry = execFileSync(
    "git",
    ["--no-replace-objects", "ls-tree", "-z", currentHead, "--", file],
    { cwd: rootPath },
  );
  const blob = createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
  assert.deepEqual(gitEntry, Buffer.from(`100644 blob ${blob}\t${file}\0`));
  return {
    file,
    sha256: sha(body),
    byteLength: body.length,
    gitEntryHex: gitEntry.toString("hex"),
  };
});

const priorBytes = read("tests/_fixtures/ci/owned-corsa-source-before.json");
assert.equal(sha(priorBytes), "80fce9f7a5737d4aaf708911310b0e07a9f52801e87d8ffb7f56170c45e4747b");
const prior = JSON.parse(priorBytes);
assert.equal(prior.source, ownedCorsaSource.baseline);
const before = new Map(
  prior.files.map((entry) => [
    entry.file,
    {
      mode: entry.mode,
      blob: entry.blob,
      body: Buffer.from(entry.rawBase64, "base64"),
    },
  ]),
);
const callerMetadata = JSON.parse(
  read("tools/support/compat/github/owned-corsa-caller-metadata.json"),
);
for (const entry of callerMetadata.files)
  before.set(
    entry.file,
    entry.actualBefore
      ? {
          mode: entry.actualBefore.mode,
          blob: entry.actualBefore.blob,
          body: Buffer.from(entry.actualBefore.rawBase64, "base64"),
        }
      : null,
  );
const after = new Map(
  copied.map((entry) => {
    const body = read(entry.file);
    return [
      entry.file,
      {
        mode: "100644",
        body,
        blob: createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      },
    ];
  }),
);
const project = (entries) =>
  projectOwnedCorsaLexicalCopied({
    copied: entries,
    originalPaths: originalSourcePaths(manifest, helperPaths),
    baseline: ownedCorsaSource.baseline,
    production: [
      ...ownedCorsaSource.rows.map(({ file }) => file),
      ...nativeProxyManifest.files.map(([file]) => file),
    ],
    alreadyQualified: new Set(nativeProxyManifest.files.map(([file]) => file)),
    historicalPairs: nativeLexicalCompositionBodyPairs,
    reader: (side, file) => (side === "before" ? before : after).get(file) ?? null,
  });
const actualView = project(copied);
// The existing source-first helper authenticates237 +2 callers +119 frames.
// Only this named whole pnpm source cell is historical; copied remains actual.
function preservedSourceView(view) {
  const file = "pnpm-lock.yaml";
  const historical = historicalCompositionBody("sdk", file);
  assert.equal(sha(historical), "12e66667fc82ac9a5b72d1e9ab2aa59f7c1466ea8be689f17a67678bf31a0915");
  const actual = view.historicalReader(file);
  assert.equal(sha(actual), "f83208b48ac69a7fd603384bb4962d0da0116b04ff2b19793daa2a63dc5378db");
  const entry = view.historicalCopied.find((entry) => entry.file === file);
  assert.equal(entry.sha256, sha(actual));
  assert.equal(entry.byteLength, actual.length);
  const blob = createHash("sha1")
    .update(`blob ${historical.length}\0`)
    .update(historical)
    .digest("hex");
  return {
    ...view,
    historicalCopied: view.historicalCopied.map((entry) =>
      entry.file === file
        ? {
            ...entry,
            sha256: sha(historical),
            byteLength: historical.length,
            gitEntryHex: Buffer.from(`100644 blob ${blob}\t${file}\0`).toString("hex"),
          }
        : entry,
    ),
    historicalReader: (name) =>
      name === file ? Buffer.from(historical) : view.historicalReader(name),
  };
}
const preservedView = preservedSourceView(actualView);
const validateCurrentCopied = (manifest, entries) => {
  const view = preservedSourceView(project(entries));
  return validateLexicalSourceBodies(manifest, view.historicalCopied, view.historicalReader);
};

await runOwnedCargoOriginalRefusal({ actualView, copied, old, manifest, after });

await test("authenticated whole incoming recipe source retains the exact original caller refusal and finite successor", () => {
  let refusal;
  try {
    assert.equal(currentSHA, oldSHA, caller);
  } catch (error) {
    refusal = error;
  }
  assert.throws(() => old.validateLexicalSourceBodies(manifest, preservedView.historicalCopied), {
    name: refusal.name,
    code: refusal.code,
    message: refusal.message,
    actual: refusal.actual,
    expected: refusal.expected,
    operator: refusal.operator,
    generatedMessage: refusal.generatedMessage,
  });
  const result = validateCurrentCopied(manifest, copied);
  assert.equal(result.completeSdkProductionBodies, 121);
  assert.equal(result.incomingLexicalSourceBodies, 9);
  assert.equal(result.incomingLexicalCorpusBodies, 21);
  assert.equal(result.originalRuntimeBodies, 23);
  assert.equal(result.finalGuardBodies, 2);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
  assert.equal(result.finalSource.previousAfterSHA256, oldSHA);
  assert.equal(result.finalSource.afterSHA256, currentSHA);
  assert.equal(result.finalSource.completeFixtureBodies, 7);
});

await test("retained historical final source and original lexical fastpath remain separately qualified", () => {
  // This explicit historical tuple is not the actual current caller observation.
  const historical = preservedView.historicalCopied.map((entry) =>
    entry.file === caller ? { ...entry, sha256: oldSHA } : entry,
  );
  assert.deepEqual(
    validateLexicalSourceBodies(manifest, historical),
    old.validateLexicalSourceBodies(manifest, historical),
  );
  const originalBodies = new Map(
    [...manifest.files, ...manifest.corpusBodies, ...manifest.preservedBodies].map((row) => [
      row[0],
      row.at(-1),
    ]),
  );
  const lexical = actualView.historicalCopied.map((entry) =>
    originalBodies.has(entry.file) ? { ...entry, sha256: originalBodies.get(entry.file) } : entry,
  );
  assert.equal(validateLexicalSourceBodies(manifest, lexical), null);
  assert.equal(old.validateLexicalSourceBodies(manifest, lexical), null);
});

await test("complete current copied census refuses missing duplicate unknown and foreign bodies", () => {
  for (const { file } of copied) {
    assert.throws(
      () =>
        validateCurrentCopied(
          manifest,
          copied.filter((entry) => entry.file !== file),
        ),
      undefined,
      file,
    );
    assert.throws(
      () =>
        validateCurrentCopied(manifest, [...copied, copied.find((entry) => entry.file === file)]),
      undefined,
      file,
    );
    assert.throws(
      () =>
        validateCurrentCopied(
          manifest,
          copied.map((entry) =>
            entry.file === file ? { ...entry, sha256: "f".repeat(64) } : entry,
          ),
        ),
      undefined,
      file,
    );
  }
  assert.throws(() =>
    validateCurrentCopied(manifest, [
      ...copied,
      { file: "crates/vize_canon/src/unknown.rs", sha256: "f".repeat(64) },
    ]),
  );
  assert.equal(new Set(sourcePaths(manifest)).size, sourcePaths(manifest).length);
  for (const file of lexicalFinalSourcePaths)
    assert.equal(copied.filter((entry) => entry.file === file).length, 1);
});

await test("whole caller inverse capsule helper metadata and every fixture member refuse physical-byte changes", () => {
  for (const file of [caller, ...lexicalFinalSourcePaths]) {
    const originalBody = actualView.historicalReader(file);
    const changed = Buffer.from(originalBody);
    changed[0] ^= 1;
    assert.throws(
      () =>
        validateLexicalFinalSource(actualView.historicalCopied, (name) =>
          name === file ? changed : actualView.historicalReader(name),
        ),
      undefined,
      file,
    );
    assert.throws(
      () =>
        validateLexicalFinalSource(actualView.historicalCopied, (name) =>
          name === file ? originalBody.subarray(1) : actualView.historicalReader(name),
        ),
      undefined,
      file,
    );
  }
  assert.deepEqual(
    validateLexicalFinalSource(actualView.historicalCopied, actualView.historicalReader),
    validateLexicalSourceBodies(
      manifest,
      preservedView.historicalCopied,
      preservedView.historicalReader,
    ).finalSource,
  );
});

await test("the one complete additional plan body qualifies before projection and refuses absent foreign mode blob or partial inputs", () => {
  const file = "tools/support/compat/github/native-lexical-dollar-plan.mjs";
  const actual = after.get(file);
  assert(actual);
  for (const changed of [
    null,
    { ...actual, mode: "100755" },
    { ...actual, mode: "120000" },
    { ...actual, blob: "f".repeat(40) },
    { ...actual, body: actual.body.subarray(1) },
    { ...actual, body: Buffer.concat([actual.body, Buffer.from(" ")]) },
  ]) {
    const calls = [];
    assert.throws(() =>
      projectOwnedCorsaLexicalCopied({
        copied,
        originalPaths: originalSourcePaths(manifest, helperPaths),
        baseline: ownedCorsaSource.baseline,
        production: [
          ...ownedCorsaSource.rows.map(({ file }) => file),
          ...nativeProxyManifest.files.map(([file]) => file),
        ],
        alreadyQualified: new Set(nativeProxyManifest.files.map(([file]) => file)),
        historicalPairs: nativeLexicalCompositionBodyPairs,
        reader: (side, name) => {
          calls.push([side, name]);
          return side === "after" && name === file
            ? changed
            : ((side === "before" ? before : after).get(name) ?? null);
        },
      }),
    );
    assert.deepEqual(calls, [
      ["after", "tools/support/compat/github/native-lexical-composition-custody.mjs"],
      ["after", file],
    ]);
  }
});

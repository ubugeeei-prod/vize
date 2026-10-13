import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { javascriptSfcSourcePaths } from "./javascript-sfc-source-context.mjs";
import {
  authenticateOwnedJavaScriptSource,
  ownedJavaScriptLexicalPaths,
} from "./owned-corsa-javascript-source.mjs";
import { projectOwnedCorsaNativeMainAfter } from "./owned-corsa-native-main-source.mjs";
import { ownedCorsaSource, qualifyOwnedCorsaSource } from "./owned-corsa-source-custody.mjs";
import { verifyOwnedCorsaCallers, ownedCorsaCallerPaths } from "./owned-corsa-caller-custody.mjs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const metadataFile = new URL("./owned-corsa-lexical-adapter-metadata.json", import.meta.url);
const stat = fs.lstatSync(metadataFile);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(metadataFile);
assert.equal(sha(bytes), "c92a7f0a91cc67bc4a3d7e40da8baabd850ca4d02530b9da8ee9659434cf5ed2");
const adapter = JSON.parse(bytes);

export function verifyOwnedCorsaLexicalAdapter(reader) {
  const old = Buffer.from(adapter.previousRawBase64, "base64");
  assert.equal(old.toString("base64"), adapter.previousRawBase64);
  assert.equal(adapter.previousMode, "100644");
  assert.equal(old.length, adapter.previousBytes);
  assert.equal(sha(old), adapter.previousSHA256);
  assert.equal(
    createHash("sha1").update(`blob ${old.length}\0`).update(old).digest("hex"),
    adapter.previousBlob,
  );
  const actual = reader("after", adapter.file);
  assert(actual && actual.mode === "100644");
  assert.equal(sha(actual.body), adapter.afterSHA256);
  let current = old.toString();
  for (const [before, after] of adapter.forwardClauses) {
    assert.equal(current.split(before).length, 2);
    current = current.replace(before, after);
  }
  assert.deepEqual(
    Buffer.from(current),
    actual.body,
    "whole original lexical helper, four reader-only clauses and exact formatter successor",
  );
  assert.deepEqual(
    adapter.additionalBodies.map(({ file }) => file),
    ["tools/support/compat/github/native-lexical-dollar-plan.mjs"],
  );
  for (const expected of adapter.additionalBodies) {
    const body = reader("after", expected.file);
    assert(body && body.mode === "100644" && Buffer.isBuffer(body.body));
    assert.equal(body.blob, expected.blob);
    assert.equal(
      createHash("sha1").update(`blob ${body.body.length}\0`).update(body.body).digest("hex"),
      expected.blob,
    );
    assert.equal(body.body.length, expected.bytes);
    assert.equal(sha(body.body), expected.sha256);
  }
}

const prefix = "tools/support/compat/github/";
export const ownedCorsaSupplementPaths = [
  ...ownedJavaScriptLexicalPaths,
  ...[
    "owned-corsa-current-baseline.mjs",
    "owned-corsa-native-main-source.mjs",
    "owned-corsa-native-main-source.json",
    "owned-corsa-test-source.mjs",
    "owned-corsa-test-source.json",
  ].map((name) => prefix + name),
  prefix + "native-lexical-dollar-plan.mjs",
  ...ownedCorsaCallerPaths,
  prefix + "owned-corsa-source-custody.mjs",
  prefix + "owned-corsa-source-reader.mjs",
  prefix + "owned-corsa-clippy-source.mjs",
  prefix + "owned-corsa-clippy-source.json",
  prefix + "owned-corsa-package-source.mjs",
  prefix + "owned-corsa-source-manifest.json",
  ...ownedCorsaSource.parts.map(({ file }) => prefix + file),
  prefix + "owned-corsa-caller-custody.mjs",
  prefix + "owned-corsa-caller-metadata.json",
  prefix + "owned-corsa-lexical-source.mjs",
  prefix + "owned-corsa-lexical-metadata.mjs",
  prefix + "owned-corsa-lexical-adapter-metadata.json",
  prefix + "owned-corsa-package-custody.py",
  prefix + "owned-corsa-package-consumer.sh",
  prefix + "owned-canon-external-consumer.mjs",
  "tests/_fixtures/ci/owned-corsa-package-consumer/Cargo.toml",
  "tests/_fixtures/ci/owned-corsa-package-consumer/src/lib.rs",
  "tests/_fixtures/ci/owned-canon-external-consumer/lib.rs",
  "tests/performance/support/warm-type-backed-owned-corsa-host.ts",
  "tests/performance/support/warm-type-backed-owned-corsa-recipe.ts",
];

export function ownedCorsaLexicalPaths(original) {
  return [
    ...new Set([
      ...original,
      ...ownedCorsaSource.rows.map(({ file }) => file),
      ...ownedCorsaSupplementPaths,
    ]),
  ];
}

/** The original producer's copied array stays immutable; this is a named counterfactual view. */
export function projectOwnedCorsaLexicalCopied({
  copied,
  originalPaths,
  baseline,
  production,
  alreadyQualified,
  reader,
  historicalPairs,
}) {
  verifyOwnedCorsaLexicalAdapter(reader);
  const graph = qualifyOwnedCorsaSource({
    baseline,
    production,
    alreadyQualified,
    reader,
    historicalPairs,
  });
  assert(graph, "the complete actual237 source pair must qualify before any lexical projection");
  const callers = verifyOwnedCorsaCallers(reader);
  assert.equal(new Set(copied.map(({ file }) => file)).size, copied.length);
  assert.deepEqual(
    copied.map(({ file }) => file).toSorted((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
    ownedCorsaLexicalPaths(originalPaths).toSorted((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
  );
  const sha = (body) => createHash("sha256").update(body).digest("hex");
  for (const entry of copied) {
    const actual = reader("after", entry.file);
    assert(actual && actual.mode === "100644");
    assert.equal(
      entry.gitEntryHex,
      Buffer.from(`100644 blob ${actual.blob}\t${entry.file}\0`).toString("hex"),
    );
    assert.equal(entry.byteLength, actual.body.length);
    assert.equal(entry.sha256, sha(actual.body));
  }
  const javascript = authenticateOwnedJavaScriptSource(reader);
  const mainReader = projectOwnedCorsaNativeMainAfter(reader);
  const original = new Set(ownedCorsaSource.rows.map(({ file }) => file));
  const old = new Set(originalPaths);
  const source = (file) => {
    const caller = callers.reader("after", file);
    const graphBody = graph.historicalReader("after", file);
    const mainBody = mainReader(file);
    const jsBody =
      javascriptSfcSourcePaths.includes(file) && !original.has(file)
        ? file === "tests/_fixtures/differential/typechecker/manifest.json"
          ? javascript.retainedOwnedAfter(file)
          : javascript.historicalAfter(file)
        : null;
    const value =
      caller?.blob !== reader("after", file)?.blob
        ? caller
        : mainBody?.blob !== reader("after", file)?.blob
          ? mainBody
          : jsBody
            ? {
                mode: "100644",
                blob: createHash("sha1")
                  .update(`blob ${jsBody.length}\0`)
                  .update(jsBody)
                  .digest("hex"),
                body: jsBody,
              }
            : graphBody;
    assert(value);
    return { ...value, body: Buffer.from(value.body) };
  };
  const historicalCopied = copied
    .filter(({ file }) => old.has(file))
    .map((entry) => {
      const value = source(entry.file);
      return {
        ...entry,
        gitEntryHex: Buffer.from(`100644 blob ${value.blob}\t${entry.file}\0`).toString("hex"),
        byteLength: value.body.length,
        sha256: sha(value.body),
      };
    });
  return {
    receipt: {
      authority: "complete-reviewed-owned-corsa-lexical-source-counterfactual",
      sourceManifestSHA256: graph.manifestSHA256,
      completeProductionBodies: 237,
      originalProductionBodies: 121,
      newArchiveMemberBodies: 112,
      additiveLicenseBodies: 4,
      completeActualCopiedBodies: copied.length,
      completeHistoricalCopiedBodies: historicalCopied.length,
      callerSource: callers.receipt,
      producerCopiedIdentityChanged: false,
      runtimeCredit: false,
      performanceCredit: false,
    },
    historicalCopied,
    historicalReader: (file) => source(file).body,
  };
}

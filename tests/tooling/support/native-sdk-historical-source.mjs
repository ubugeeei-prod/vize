// Test-only whole historical source loans; this grants no native or performance credit.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { ownedCorsaReader } from "../../../tools/support/compat/github/owned-corsa-source-reader.mjs";
import {
  ownedCorsaSource,
  qualifyOwnedCorsaSource,
} from "../../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import {
  ownedCorsaCallerPaths,
  verifyOwnedCorsaCallers,
} from "../../../tools/support/compat/github/owned-corsa-caller-custody.mjs";
import { verifyOwnedCorsaLexicalAdapter } from "../../../tools/support/compat/github/owned-corsa-lexical-source.mjs";
import { authenticateOwnedCorsaAfterContext } from "../../../tools/support/compat/github/owned-corsa-clippy-source.mjs";
import { nativeLexicalCompositionBodyPairs } from "../../performance/support/warm-type-backed-native-lexical-composition-host.ts";
import { ensureHostedLiteralSource } from "../../performance/support/warm-type-backed-git-reader-literal-source.ts";
import { historicalCompositionBody as retainedCompositionBody } from "./native-lexical-composition-source.ts";
import { authenticateHistoricalIncomingLock } from "./native-sdk-incoming-lock.mjs";
import { ownedCorsaSdkRepairFrames } from "./owned-corsa-sdk-repair-source.mjs";
import { ownedCorsaTestSourceIdentities } from "../../../tools/support/compat/github/owned-corsa-test-source.mjs";
export const nativeSdkSourceRoot = fs.realpathSync(new URL("../../../", import.meta.url));
const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const capsulePath = new URL("../fixtures/native-sdk-historical-source.json", import.meta.url);
const capsuleStat = fs.lstatSync(capsulePath);
assert(capsuleStat.isFile() && (capsuleStat.mode & 0o777) === 0o644);
const capsuleBytes = fs.readFileSync(capsulePath);
assert.equal(sha(capsuleBytes), "899d7c384501209b2322b64416cb4f7be6a9532a46aed6c11e7cc0e7eb5e87ab");
const capsule = JSON.parse(capsuleBytes);
assert.equal(capsule.version, 1);
assert.equal(capsule.selectedSource, "b2caebeab6fb5bc817b4e31a65d5d560449b691b");
assert.equal(capsule.actualBeforeSource, "2489128d5f8fab1b7442a1e5c09b4e1701cbf185");
assert.equal(capsule.historicalRuntimeSource, "8304eb7a08a96e58ca0a318cc2fa6ffc6e9acd75");
assert.equal(capsule.completeActualProductionBodies, 237);
assert.equal(capsule.completeActualCallerBodies, 2);
assert.equal(capsule.runtimeCredit, false);
assert.equal(capsule.performanceCredit, false);
assert.equal(capsule.historicalSourcePaths.length, 206);
assert.equal(new Set(capsule.historicalSourcePaths).size, 206);
const fileNames = (entries) => entries.map(({ file }) => file);
const runtimeNames = [
  "native-lexical-dollar.sh",
  "native-lexical-dollar-metadata.mjs",
  "native-lexical-dollar-receipt.mjs",
  "native-lexical-composition-custody.mjs",
].map((name) => "tools/support/compat/github/" + name);
assert.deepEqual(capsule.historicalRuntimeModulePaths, runtimeNames);
assert.deepEqual(fileNames(capsule.historicalRuntimeBodies), runtimeNames);
const archived = new Map();
for (const entry of capsule.historicalRuntimeBodies) {
  assert.equal(entry.source, capsule.historicalRuntimeSource);
  assert.equal(entry.mode, "100644");
  const body = Buffer.from(entry.rawBase64, "base64");
  assert.equal(body.toString("base64"), entry.rawBase64);
  assert.equal(body.length, entry.bytes);
  assert.equal(sha(body), entry.sha256);
  assert.equal(blob(body), entry.blob);
  assert.equal(body.toString().split("import.meta.url").length - 1, entry.importMetaOccurrences);
  archived.set(entry.file, { ...entry, body });
}
const frames = new Map(capsule.currentFrames.map((entry) => [entry.file, entry]));
assert.equal(frames.size, capsule.currentFrames.length);
for (const entry of frames.values()) {
  assert.equal(entry.source, capsule.selectedSource);
  assert.equal(entry.mode, "100644");
}
const finalContextURL = new URL("./native-sdk-final-source-context.json", import.meta.url);
const finalContextStat = fs.lstatSync(finalContextURL);
assert(finalContextStat.isFile() && (finalContextStat.mode & 0o777) === 0o644);
const finalContextBytes = fs.readFileSync(finalContextURL);
assert.equal(
  sha(finalContextBytes),
  "c9e0b29dfb08d7e4a933ebe7a25281fb54cc99e418f0417ed9800e8b2348edea",
);
const finalContext = JSON.parse(finalContextBytes);
assert.equal(finalContext.schema, 1);
assert.equal(finalContext.originalFrameSHA256, sha(capsuleBytes));
assert.equal(finalContext.runtimeCredit, false);
assert.equal(finalContext.performanceCredit, false);
const replacements = [
  "tools/support/compat/github/owned-corsa-source-custody.mjs",
  "tools/support/compat/github/owned-corsa-lexical-source.mjs",
  "pnpm-lock.yaml",
];
const additions = [
  "tools/support/compat/github/owned-corsa-clippy-source.mjs",
  "tools/support/compat/github/owned-corsa-clippy-source.json",
  "tests/tooling/sdk-ascii-symbol-original-callers.test.ts",
  "tests/tooling/sdk-component-join-source.test.mjs",
  "tests/tooling/support/sdk-ascii-symbol-original-callers.ts",
  "tests/tooling/support/sdk-component-join-files.json",
  "tests/tooling/support/sdk-component-join-replay.mjs",
  "tests/tooling/support/sdk-component-join.json",
  "tests/tooling/support/sdk-component-join.mjs",
];
assert.deepEqual(fileNames(finalContext.replacementFrames), replacements);
assert.deepEqual(fileNames(finalContext.additionalFrames), additions);
for (const entry of finalContext.replacementFrames) {
  assert.deepEqual(entry.before, frames.get(entry.file), "whole original117 frame stays exact");
  frames.set(entry.file, entry.after);
}
for (const entry of finalContext.additionalFrames) {
  assert(!frames.has(entry.file));
  assert.equal(entry.before, null);
  frames.set(entry.file, entry.after);
}
for (const entry of frames.values()) {
  assert.equal(entry.mode, "100644");
  assert([capsule.selectedSource, finalContext.preparationSource].includes(entry.source));
}
const provisioningNames = [
  "tests/performance/support/warm-type-backed-git-reader-literal-source.ts",
  "tests/performance/support/warm-type-backed-git-reader-nodeep.ts",
];
assert.deepEqual(fileNames(finalContext.hostedProvisioningFrames), provisioningNames);
const lockBodies = authenticateHistoricalIncomingLock(finalContext, capsule.currentFrames);
const production = ownedCorsaSource.rows.map(({ file }) => file);
const known = new Set([...production, ...ownedCorsaCallerPaths, ...frames.keys()]);
for (const file of capsule.historicalSourcePaths) assert(known.has(file), file);
const cargo = [
  "Cargo.toml",
  "Cargo.lock",
  "vendor/corsa_core/Cargo.toml",
  "vendor/corsa_lsp/Cargo.toml",
];
const loans = new Set([...cargo, ...ownedCorsaCallerPaths, ...runtimeNames, "pnpm-lock.yaml"]);
const historicalEntries = new Set([
  "sdk:crates/vize_canon/src/batch/executor/cli.rs",
  "sdk:crates/vize_canon/src/batch/executor/session.rs",
  "sdk:tests/performance/support/warm-type-backed-native-terminal-final.ts",
  "lexical:crates/vize_canon/src/batch/executor/cli.rs",
  "lexical:crates/vize_canon/src/batch/executor/session.rs",
  "lexical:tests/_fixtures/differential/typechecker/manifest.json",
]);
const clone = (value) => value && { ...value, body: Buffer.from(value.body) };
const rawReaders = new Map();
function currentReader(root) {
  for (const expected of finalContext.hostedProvisioningFrames) {
    const target = path.join(root, expected.file),
      stat = fs.lstatSync(target);
    assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
    const body = fs.readFileSync(target);
    assert.equal(body.length, expected.bytes);
    assert.equal(sha(body), expected.sha256);
    assert.equal(blob(body), expected.blob);
  }
  ensureHostedLiteralSource(root, capsule.actualBeforeSource);
  ensureHostedLiteralSource(root, finalContext.preparationSource);
  const head = execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], { cwd: root })
    .toString()
    .trim();
  const key = root + ":" + head;
  if (!rawReaders.has(key))
    rawReaders.set(key, ownedCorsaReader(root, capsule.actualBeforeSource, head));
  return { head, reader: rawReaders.get(key) };
}
/** Supplied callbacks must equal genuine raw Git and current physical whole bodies. */
export function createNativeSdkHistoricalView(root = nativeSdkSourceRoot, suppliedReader) {
  root = fs.realpathSync(root);
  const { head, reader } = currentReader(root);
  const git = (args) => execFileSync("git", ["--no-replace-objects", ...args], { cwd: root });
  assert.match(finalContext.preparationSource, /^[a-f0-9]{40}$/u);
  const rawCommit = git(["cat-file", "commit", finalContext.preparationSource]);
  assert.equal(rawCommit.length, finalContext.rawCommitBytes);
  assert.equal(sha(rawCommit), finalContext.rawCommitSHA256);
  assert.equal(rawCommit.toString().split("\n")[0], "tree " + finalContext.preparationTree);
  // A captured source tuple and complete current bodies grant no ancestry or squash credit.
  for (const expected of finalContext.hostedProvisioningFrames) {
    const entry = git(["ls-tree", "-z", head, "--", expected.file]);
    assert.deepEqual(entry, Buffer.from(`100644 blob ${expected.blob}\t${expected.file}\0`));
    assert.deepEqual(
      git(["cat-file", "blob", expected.blob]),
      fs.readFileSync(path.join(root, expected.file)),
    );
  }
  const actual = (side, file) => {
    assert(["before", "after"].includes(side));
    assert(known.has(file), "unknown historical source refuses: " + file);
    const real = reader(side, file);
    if (side === "after" && real) {
      const target = path.join(root, file);
      const stat = fs.lstatSync(target);
      assert(stat.isFile() && (stat.mode & 0o777) === 0o644, "regular current source: " + file);
      assert.deepEqual(fs.readFileSync(target), real.body, "current whole source: " + file);
    }
    if (suppliedReader)
      assert.deepEqual(
        suppliedReader(side, file),
        real,
        "supplied whole source equals raw Git: " + file,
      );
    return clone(real);
  };
  for (const { file } of finalContext.additionalFrames)
    assert.equal(
      actual("before", file),
      null,
      "genuine before absence for the two new policy bodies: " + file,
    );
  for (const [file, expected] of ownedCorsaSdkRepairFrames(frames, actual)) {
    const value = actual("after", file);
    assert(value && value.mode === expected.mode);
    assert.equal(value.blob, expected.blob);
    assert.equal(value.body.length, expected.bytes);
    assert.equal(sha(value.body), expected.sha256, "complete current frame: " + file);
    assert.equal(blob(value.body), expected.blob);
  }
  const graph = qualifyOwnedCorsaSource({
    baseline: capsule.actualBeforeSource,
    production,
    alreadyQualified: new Set(),
    reader: actual,
    historicalPairs: nativeLexicalCompositionBodyPairs,
  });
  assert(graph, "complete actual237 before/after source is mandatory before a historical loan");
  const callers = verifyOwnedCorsaCallers(actual);
  verifyOwnedCorsaLexicalAdapter(actual);
  const currentAfter = authenticateOwnedCorsaAfterContext(ownedCorsaSource.rows, (file) =>
    actual("after", file),
  );
  assert(
    currentAfter?.context,
    "the independently qualified current six-manifest class is mandatory",
  );
  assert.equal(
    currentAfter.context.sourceContextSHA256,
    "9b9212acd2ae6a56c4c4a8fb86abec5e550a2a55347ae030a896785e77f8c96f",
  );
  const policy = JSON.parse(actual("after", additions[1]).body);
  assert.equal(currentAfter.context.wholeCorrectedRustBodies, 2);
  assert.deepEqual(currentAfter.context.correctedSourceFiles, [
    "vendor/corsa_client/src/api/handles.rs",
    "vendor/corsa/src/bin/bench_real_corsa/args.rs",
  ]);
  const currentCells = new Map(
    [...policy.entries, ...policy.sourceCorrections].map(({ file, currentAfter }) => [
      file,
      currentAfter,
    ]),
  );
  assert.equal(currentCells.size, 8);
  const currentAfterIdentities = ownedCorsaSource.rows.map((row) => {
    const expected =
      ownedCorsaTestSourceIdentities.get(row.file) ?? currentCells.get(row.file) ?? row.after;
    const value = currentAfter.actualAfter(row.file);
    assert(value && value.mode === expected.mode);
    assert.equal(value.blob, expected.blob);
    assert.equal(value.body.length, expected.bytes);
    assert.equal(sha(value.body), expected.sha256);
    assert.equal(blob(value.body), expected.blob);
    return Object.freeze({
      file: row.file,
      identity: Object.freeze({
        mode: expected.mode,
        blob: expected.blob,
        bytes: expected.bytes,
        sha256: expected.sha256,
      }),
    });
  });
  const historicalReader = (side, file) => {
    const current = actual(side, file);
    if (side === "after" && file === "pnpm-lock.yaml") {
      assert.deepEqual(
        current.body,
        lockBodies.actualIncomingAfter.body,
        "whole actual incoming lock before historical loan",
      );
      return clone(lockBodies.retainedHistoricalAfter);
    }
    if (side === "after" && archived.has(file)) return clone(archived.get(file));
    if (ownedCorsaCallerPaths.includes(file)) return callers.reader(side, file);
    if (cargo.includes(file)) return graph.historicalReader(side, file);
    return current;
  };
  return Object.freeze({
    receipt: Object.freeze({
      authority: "test-only-complete-current-sdk-and-historical-source",
      baseline: capsule.actualBeforeSource,
      head,
      completeProductionBodies: 237,
      completeCallerBodies: 2,
      completeCurrentFrameBodies: frames.size,
      originalCurrentFrameBodies: 117,
      finiteReplacementFrameBodies: 3,
      finiteAdditionalFrameBodies: 9,
      hostedProvisioningBodies: 2,
      wholeHistoricalLockBodies: 1,
      ancestryCredit: false,
      squashCredit: false,
      dependencyCredit: false,
      toolchainCredit: false,
      preparationSource: finalContext.preparationSource,
      currentAfterContext: Object.freeze({ ...currentAfter.context }),
      originalProductionBodies: 121,
      historicalRuntimeModules: 4,
      historicalSourceBodies: 206,
      historicalRuntimeSource: capsule.historicalRuntimeSource,
      runtimeCredit: false,
      performanceCredit: false,
    }),
    actualReader: actual,
    currentAfterIdentities: Object.freeze(currentAfterIdentities),
    historicalReader,
    body: (file) => {
      const value = historicalReader("after", file);
      assert(value, "genuine current absence cannot become a historical body: " + file);
      return Buffer.from(value.body);
    },
  });
}
let retainedView;
export function nativeSdkHistoricalView() {
  return (retainedView ??= createNativeSdkHistoricalView());
}
export function archivedNativeSdkModule(file) {
  nativeSdkHistoricalView();
  assert(archived.has(file), "unknown historical runtime module refuses: " + file);
  return clone(archived.get(file));
}
export function historicalCompositionBody(authority, file) {
  assert(["sdk", "lexical"].includes(authority));
  const view = nativeSdkHistoricalView();
  view.actualReader("after", file);
  return historicalEntries.has(authority + ":" + file)
    ? Buffer.from(retainedCompositionBody(authority, file))
    : view.body(file);
}
const sourceRead = (target, options) => {
  const file = target instanceof URL ? fileURLToPath(target) : target;
  if (typeof file === "string" && path.isAbsolute(file)) {
    const relative = path.relative(nativeSdkSourceRoot, file);
    if (loans.has(relative)) {
      const body = nativeSdkHistoricalView().body(relative);
      const encoding = typeof options === "string" ? options : options?.encoding;
      return encoding ? body.toString(encoding) : body;
    }
  }
  return fs.readFileSync(target, options);
};
export default new Proxy(fs, {
  get: (target, property) =>
    property === "readFileSync" ? sourceRead : Reflect.get(target, property),
});

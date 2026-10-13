import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  authenticateNoMutatingPropsSourceFamily,
  composeNoMutatingHistoricalLaw,
  composeNoMutatingSourceHelper,
  noMutatingCustodyDirectory,
  noMutatingSfcCarrier,
  reviewedNoMutatingHelperHistoryReadView,
  reviewedNoMutatingSourceReadView,
} from "./support/no-mutating-props-source-custody.mjs";
import { authenticateDeepPropsSourceFamily } from "./support/props-deep-source-custody.mjs";
import { authenticateRuntimeApiSourceFamily } from "./support/runtime-props-api-source-custody.mjs";
import { authenticateSetupExportPresetSourceFamily } from "./support/setup-export-preset-source-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (name) => fs.readFileSync(path.join(root, name));
const hash = (body) => createHash("sha256").update(body).digest("hex");

await test("the current whole family keeps eleven bodies and the literal failed histories", () => {
  const current = authenticateNoMutatingPropsSourceFamily(root, read);
  assert.equal(current.sources.length, 12);
  assert.deepEqual(current.changedPaths, [noMutatingSfcCarrier]);
  assert.equal(current.registeredRuleBodies.length, 11);
  const runtime = authenticateRuntimeApiSourceFamily(root, read);
  const preset = authenticateSetupExportPresetSourceFamily(root, runtime, read);
  const depth = authenticateDeepPropsSourceFamily(root, read);
  assert.equal(runtime.sources.length, 12);
  assert.equal(preset.sources.length, 12);
  assert.equal(
    depth.currentSuccessor.currentSuccessor.incomingFamily.sha256,
    "39cf7ab7633378ab462c6af46443fa0e235b070d067db739eaa040760e3e996f",
  );
  for (const helper of current.originalHelpers) {
    assert.deepEqual(
      read(helper.path),
      composeNoMutatingSourceHelper(helper.path, read(helper.archive)),
    );
  }
});

await test("every current production, corpus, manifest and archive mutation remains refused", () => {
  const observed = new Set();
  const tracked = (name) => {
    observed.add(name);
    return read(name);
  };
  const current = authenticateNoMutatingPropsSourceFamily(root, tracked);
  const names = new Set(
    [
      ...observed,
      `${noMutatingCustodyDirectory}/successor.json`,
      current.incomingFamily.path,
      current.originalSourceLaw.path,
      current.originalSourceLaw.archive,
      ...current.originalHistoricalLaws.flatMap(({ path, archive }) => [path, archive]),
      ...current.sources.flatMap(({ path, before, after }) => [
        path,
        before.archive,
        after.archive,
      ]),
      ...current.registeredRuleBodies.flatMap(({ path, before, after }) => [
        path,
        before.archive,
        after.archive,
      ]),
      ...current.originalHelpers.flatMap(({ path, archive }) => [path, archive]),
    ].filter(Boolean),
  );
  for (const name of names) {
    const unknown = (requested) =>
      requested === name ? Buffer.concat([read(requested), Buffer.from(" ")]) : read(requested);
    assert.throws(() => authenticateNoMutatingPropsSourceFamily(root, unknown), name);
    assert.throws(() => authenticateDeepPropsSourceFamily(root, unknown), name);
    assert.throws(() => authenticateRuntimeApiSourceFamily(root, unknown), name);
  }
  const sfc = current.sources.find(({ changed }) => changed);
  const historical = (name) =>
    name === noMutatingSfcCarrier ? read(sfc.before.archive) : read(name);
  assert.throws(() => authenticateNoMutatingPropsSourceFamily(root, historical));
  assert.equal(authenticateDeepPropsSourceFamily(root, historical).sources.length, 12);
});

await test("the narrow historical views forward unrelated requests and preserve all old assertions", () => {
  const current = authenticateNoMutatingPropsSourceFamily(root, read);
  const calls = [];
  const tracked = (name, ...args) => {
    calls.push([name, ...args]);
    return read(name);
  };
  const sourceView = reviewedNoMutatingSourceReadView(root, tracked);
  const sfc = current.sources.find(({ changed }) => changed);
  assert.deepEqual(sourceView(noMutatingSfcCarrier), read(sfc.before.archive));
  const arbitrary = current.sources[0].path;
  calls.length = 0;
  assert.deepEqual(sourceView(arbitrary, "unchanged", 42), read(arbitrary));
  assert.deepEqual(calls, [[arbitrary, "unchanged", 42]]);
  const helperView = reviewedNoMutatingHelperHistoryReadView(root, read);
  for (const helper of current.originalHelpers)
    assert.deepEqual(helperView(helper.path), read(helper.archive));
  assert.deepEqual(helperView(noMutatingSfcCarrier), read(noMutatingSfcCarrier));
  const original = read(current.originalSourceLaw.archive).toString();
  const live = read(current.originalSourceLaw.path).toString();
  const imported =
    'import { reviewedNoMutatingHelperHistoryReadView } from "./support/no-mutating-props-source-custody.mjs";\n';
  assert.equal(live.split(imported).length, 2);
  const added =
    "  const readHelperHistory = reviewedNoMutatingHelperHistoryReadView(root, readRoot);\n";
  assert.equal(live.split(added).length, 2);
  const restored = live
    .replace(imported, "")
    .replace(added, "")
    .replace("sha256(readHelperHistory(helper.path))", "sha256(readRoot(helper.path))");
  assert.equal(restored, original);
  assert.equal(hash(Buffer.from(restored)), current.originalSourceLaw.sha256);
  for (const law of current.originalHistoricalLaws) {
    const before = read(law.archive).toString();
    const after = read(law.path).toString();
    assert.deepEqual(read(law.path), composeNoMutatingHistoricalLaw(read(law.archive)));
    const imported =
      'import { reviewedNoMutatingHistoricalCaseReadView } from "./support/no-mutating-props-source-custody.mjs";\n';
    const context =
      "const physicalRead = (name) => fs.readFileSync(path.join(root, name));\nconst read = reviewedNoMutatingHistoricalCaseReadView(root, physicalRead);\n";
    assert.equal(after.split(imported).length, 2);
    assert.equal(after.split(context).length, 2);
    assert.equal(
      after
        .replace(imported, "")
        .replace(context, "const read = (name) => fs.readFileSync(path.join(root, name));\n"),
      before,
    );
  }
});

await test("matching bytes cannot disguise symlink, executable, missing or directory source custody", () => {
  const observed = new Set();
  const tracked = (name) => {
    observed.add(name);
    return read(name);
  };
  const current = authenticateNoMutatingPropsSourceFamily(root, tracked);
  const names = new Set(
    [
      ...observed,
      `${noMutatingCustodyDirectory}/successor.json`,
      current.incomingFamily.path,
      current.originalSourceLaw.path,
      current.originalSourceLaw.archive,
      ...current.originalHistoricalLaws.flatMap(({ path, archive }) => [path, archive]),
      ...current.sources.flatMap(({ path, before, after }) => [
        path,
        before.archive,
        after.archive,
      ]),
      ...current.registeredRuleBodies.flatMap(({ path, before, after }) => [
        path,
        before.archive,
        after.archive,
      ]),
      ...current.originalHelpers.flatMap(({ path, archive }) => [path, archive]),
    ].filter(Boolean),
  );
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-source-custody-"));
  try {
    for (const name of names) {
      const target = path.join(owned, name);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, read(name), { mode: 0o644 });
    }
    authenticateNoMutatingPropsSourceFamily(owned, read);
    const target = path.join(owned, noMutatingSfcCarrier);
    fs.chmodSync(target, 0o755);
    assert.throws(() => authenticateNoMutatingPropsSourceFamily(owned, read));
    fs.chmodSync(target, 0o644);
    fs.unlinkSync(target);
    assert.throws(() => authenticateNoMutatingPropsSourceFamily(owned, read));
    fs.symlinkSync(path.join(root, noMutatingSfcCarrier), target);
    assert.throws(() => authenticateNoMutatingPropsSourceFamily(owned, read));
    fs.unlinkSync(target);
    fs.mkdirSync(target);
    assert.throws(() => authenticateNoMutatingPropsSourceFamily(owned, read));
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
});

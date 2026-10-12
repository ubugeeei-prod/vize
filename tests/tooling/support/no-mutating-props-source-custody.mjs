import { reviewedNoMutatingModuleLayoutReadView } from "./no-mutating-props-module-layout-source-custody.mjs";
import { reviewedNoMutatingTestCorrectionReadView } from "./no-mutating-props-test-correction-source-custody.mjs";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { authenticateDeepPropsSourceFamily } from "./props-deep-source-custody.mjs";

export const noMutatingCustodyDirectory =
  "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody";
export const noMutatingSfcCarrier = "crates/vize_patina/src/linter/script_rules/sfc.rs";
const digest = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const matches = (body, identity) =>
  body.length === identity.bytes && digest(body) === identity.sha256;
const reader = (root) => (name) => fs.readFileSync(path.join(root, name));
const originalSfc = "cce4c13e30a66479b42ea078dc9f799afc81bbfb8e9d5464f78b75b874c42dc1";
const moduleImport =
  'import { reviewedNoMutatingSourceReadView } from "./no-mutating-props-source-custody.mjs";\n';
const selectedRead = "  read = reviewedNoMutatingSourceReadView(root, read);\n";
const helperEntries = {
  "props-deep-source-custody.mjs": [
    "authenticateDeepPropsSourceFamily(root, read = reader(root))",
    "authenticateDeepPropsImportCorrection(root, incoming, read = reader(root))",
    "authenticateDeepPropsTraitScopeCorrection(root, incoming, read = reader(root))",
  ],
  "runtime-props-api-source-custody.mjs": [
    "authenticateRuntimeApiSourceFamily(root, read = reader(root))",
    "assertReservedApiOrderSource(root, source, read = reader(root))",
  ],
  "setup-export-preset-source-custody.mjs": [
    "authenticateSetupExportPresetSourceFamily(root, incoming, read = reader(root))",
  ],
};

/** The prior helper bodies remain the entire historical predicates. */
export function composeNoMutatingSourceHelper(name, original) {
  let body = original.toString();
  const entries = helperEntries[path.posix.basename(name)];
  assert(entries, `unknown helper integration: ${name}`);
  for (const entry of entries) {
    const originalAdmission = entry.startsWith("assertReservedApiOrderSource(")
      ? '  if (matches(read(source.path), source)) return "original";\n'
      : "";
    const before = `export function ${entry} {\n${originalAdmission}`;
    assert.equal(body.split(before).length, 2);
    body = body.replace(before, before + selectedRead);
  }
  return Buffer.from(moduleImport + body);
}

export function composeNoMutatingHistoricalLaw(original) {
  const before = "const read = (name) => fs.readFileSync(path.join(root, name));\n";
  const after =
    "const physicalRead = (name) => fs.readFileSync(path.join(root, name));\nconst read = reviewedNoMutatingHistoricalCaseReadView(root, physicalRead);\n";
  const body = original.toString();
  assert.equal(body.split(before).length, 2);
  return Buffer.from(
    'import { reviewedNoMutatingHistoricalCaseReadView } from "./support/no-mutating-props-source-custody.mjs";\n' +
      body.replace(before, after),
  );
}

/** Source custody only: no measured, compiled or installed acceptance is returned. */
export function authenticateNoMutatingPropsSourceFamily(root, read = reader(root)) {
  const moduleLayoutAuthority = reviewedNoMutatingModuleLayoutReadView(root, read);
  root = moduleLayoutAuthority.root;
  read = moduleLayoutAuthority.read;
  read = reviewedNoMutatingTestCorrectionReadView(root, read);
  const bytes = read(`${noMutatingCustodyDirectory}/successor.json`);
  assert.equal(digest(bytes), "052f284b2d11f94e6d81dc68096ae74e1334c8ab4323ccaa01951e94e20a7fd1");
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, "vize.lint.no-mutating-props-sfc-source-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.base, "9e4cbaae830e5abfe1213495e9c02d2dcae941af");
  assert.deepEqual(manifest.changedPaths, [noMutatingSfcCarrier]);
  const incomingBytes = read(manifest.incomingFamily.path);
  assert(matches(incomingBytes, manifest.incomingFamily));
  assert.equal(
    digest(incomingBytes),
    "9591f993ff7ff20537ffbc532ce84af6ef201ec26e248b99d7788725a7dece59",
  );
  const incoming = JSON.parse(incomingBytes);
  assert.equal(manifest.sources.length, 12);
  assert.deepEqual(
    manifest.sources.map(({ path }) => path),
    incoming.sources.map(({ path }) => path),
  );
  assert.equal(manifest.sources.filter(({ changed }) => changed).length, 1);
  const checked = new Set([
    `${noMutatingCustodyDirectory}/successor.json`,
    manifest.incomingFamily.path,
  ]);
  const check = (name, identity) => {
    const body = read(name);
    assert(matches(body, identity), `unknown whole source body: ${name}`);
    assert.equal(blob(body), identity.gitBlob);
    checked.add(name);
    return body;
  };
  for (const [index, source] of manifest.sources.entries()) {
    assert.deepEqual(source.before, incoming.sources[index].after);
    check(source.before.archive, source.before);
    check(source.after.archive, source.after);
    check(source.path, source.after);
    if (source.path === noMutatingSfcCarrier) {
      assert.equal(source.changed, true);
      assert.equal(source.before.sha256, originalSfc);
      assert.notEqual(source.before.sha256, source.after.sha256);
    } else {
      assert.equal(source.changed, false);
      assert.deepEqual(source.after, source.before);
    }
  }
  assert.equal(manifest.registeredRuleBodies.length, 11);
  assert.equal(new Set(manifest.registeredRuleBodies.map(({ path }) => path)).size, 11);
  for (const source of manifest.registeredRuleBodies) {
    assert(!manifest.sources.some(({ path }) => path === source.path));
    if (source.before.absent) assert.deepEqual(source.before, { path: source.path, absent: true });
    else check(source.before.archive, source.before);
    check(source.after.archive, source.after);
    check(source.path, source.after);
  }
  assert.equal(manifest.originalHelpers.length, 3);
  assert.deepEqual(
    manifest.originalHelpers.map(({ path: name }) => path.posix.basename(name)),
    Object.keys(helperEntries),
  );
  for (const original of manifest.originalHelpers) {
    const before = check(original.archive, original);
    const after = read(original.path);
    assert.deepEqual(after, composeNoMutatingSourceHelper(original.path, before));
    checked.add(original.path);
  }
  const originalLaw = check(
    manifest.originalSourceLaw.archive,
    manifest.originalSourceLaw,
  ).toString();
  const sourceEntry =
    'await test("old source callers admit only the complete successor and retain whole helper history", () => {\n  const readRoot = (name) => fs.readFileSync(path.join(root, name));\n';
  assert.equal(originalLaw.split(sourceEntry).length, 2);
  const sourceImport =
    'import { reviewedNoMutatingHelperHistoryReadView } from "./support/no-mutating-props-source-custody.mjs";\n';
  const composedLaw =
    sourceImport +
    originalLaw
      .replace(
        sourceEntry,
        sourceEntry +
          "  const readHelperHistory = reviewedNoMutatingHelperHistoryReadView(root, readRoot);\n",
      )
      .replace("sha256(readRoot(helper.path))", "sha256(readHelperHistory(helper.path))");
  assert.deepEqual(read(manifest.originalSourceLaw.path), Buffer.from(composedLaw));
  checked.add(manifest.originalSourceLaw.path);
  assert.equal(manifest.originalHistoricalLaws.length, 2);
  for (const law of manifest.originalHistoricalLaws) {
    const original = check(law.archive, law);
    assert.deepEqual(read(law.path), composeNoMutatingHistoricalLaw(original));
    checked.add(law.path);
  }
  // Physical source custody refuses links/directories/executable substitutions even
  // when a supplied read callback could return an otherwise matching byte sequence.
  for (const name of checked) {
    const status = fs.lstatSync(path.join(root, name));
    assert(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111), name);
  }
  const changed = manifest.sources.find(({ changed }) => changed);
  const historicalRead = (name, ...args) => {
    const requested = name === changed.path ? changed.before.archive : name;
    const status = fs.lstatSync(path.join(root, requested));
    assert(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111), requested);
    return read(requested, ...args);
  };
  const historical = authenticateDeepPropsSourceFamily(root, historicalRead);
  assert.deepEqual(historical.currentSuccessor.currentSuccessor, incoming);
  return manifest;
}

/** Only the one declared incoming SFC body may be read by historical predicates. */
export function reviewedNoMutatingSourceReadView(root, read = reader(root)) {
  if (digest(read(noMutatingSfcCarrier)) === originalSfc) return read;
  const manifest = authenticateNoMutatingPropsSourceFamily(root, read);
  const source = manifest.sources.find(({ changed }) => changed);
  return (name, ...args) =>
    name === source.path ? read(source.before.archive, ...args) : read(name, ...args);
}

/** Retain the old whole helper integration assertion after proving current bodies. */
export function reviewedNoMutatingHelperHistoryReadView(root, read = reader(root)) {
  const manifest = authenticateNoMutatingPropsSourceFamily(root, read);
  return (name, ...args) => {
    const helper = manifest.originalHelpers.find(({ path }) => path === name);
    return helper ? read(helper.archive, ...args) : read(name, ...args);
  };
}

/** Full old cases execute under their explicitly authenticated historical context. */
export function reviewedNoMutatingHistoricalCaseReadView(root, read = reader(root)) {
  const manifest = authenticateNoMutatingPropsSourceFamily(root, read);
  const source = manifest.sources.find(({ changed }) => changed);
  return (name, ...args) => {
    const law = manifest.originalHistoricalLaws.find(({ path }) => path === name);
    if (law) return read(law.archive, ...args);
    return name === source.path ? read(source.before.archive, ...args) : read(name, ...args);
  };
}

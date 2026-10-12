import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const noMutatingTestCorrectionDirectory =
  "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/test-namespace-correction";
export const noMutatingTestCorrectionManifestSHA256 =
  "6b5aa9402ad8927e9eb68d4dcfaca99597bfa0065718bb23d7e06dbd3ebc412f";
const testPath = "crates/vize_patina/src/rules/vue/no_mutating_props/options_api/tests.rs";
const originalTestSHA = "322f83c1bc7e0a2772e3eaa1b28d9de3693aad0995715f0fe55b50aa53e40fc3";
const digest = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const reader =
  (root) =>
  (name, ...args) =>
    fs.readFileSync(path.join(root, name), ...args);
const imported =
  'import { reviewedNoMutatingTestCorrectionReadView } from "./no-mutating-props-test-correction-source-custody.mjs";\n';
const entry =
  "export function authenticateNoMutatingPropsSourceFamily(root, read = reader(root)) {\n";
const selected = "  read = reviewedNoMutatingTestCorrectionReadView(root, read);\n";

/** Entire historical predicate retained; this is its sole declared entry selection. */
export function composeNoMutatingTestCorrectionHelper(original) {
  const source = original.toString();
  assert.equal(source.split(entry).length, 2);
  return Buffer.from(imported + source.replace(entry, entry + selected));
}

/** Complete 12+11 source custody, without compiled/native/performance acceptance. */
export function authenticateNoMutatingTestCorrection(root, read = reader(root)) {
  const regular = (name) => {
    const status = fs.lstatSync(path.join(root, name));
    assert(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111), name);
  };
  const check = (name, identity) => {
    regular(name);
    const supplied = read(name);
    const physical = fs.readFileSync(path.join(root, name));
    assert.equal(supplied.length, identity.bytes, name);
    assert.equal(digest(supplied), identity.sha256, name);
    assert.equal(blob(supplied), identity.gitBlob, name);
    assert.deepEqual(physical, supplied, `physical source differs: ${name}`);
    return supplied;
  };
  const manifestPath = `${noMutatingTestCorrectionDirectory}/successor.json`;
  regular(manifestPath);
  const bytes = read(manifestPath);
  assert.equal(digest(bytes), noMutatingTestCorrectionManifestSHA256);
  assert.deepEqual(fs.readFileSync(path.join(root, manifestPath)), bytes);
  const current = JSON.parse(bytes);
  assert.equal(current.schema, "vize.lint.no-mutating-props-test-namespace-successor");
  assert.equal(current.version, 1);
  assert.equal(current.source, "dd760a4eb1299c88a1e58d4acc921a02e715844d");
  assert.deepEqual(current.familyCounts, [12, 11]);
  assert.deepEqual(current.changedPaths, [testPath]);
  assert.equal(
    current.originalManifest.sha256,
    "052f284b2d11f94e6d81dc68096ae74e1334c8ab4323ccaa01951e94e20a7fd1",
  );
  const original = JSON.parse(check(current.originalManifest.path, current.originalManifest));
  assert.equal(original.sources.length, 12);
  assert.equal(original.registeredRuleBodies.length, 11);
  const family = [...original.sources, ...original.registeredRuleBodies];
  assert.equal(new Set(family.map(({ path: name }) => name)).size, 23);
  assert.equal(current.test.path, testPath);
  const oldTest = family.find(({ path: name }) => name === testPath);
  assert.deepEqual(current.test.before, oldTest.after);
  const before = check(current.test.before.archive, current.test.before);
  const after = check(current.test.after.archive, current.test.after);
  const oldNamespace = "crate::linter::builtin_script_rules()";
  assert.equal(before.toString().split(oldNamespace).length, 2);
  assert.deepEqual(
    after,
    Buffer.from(before.toString().replace(oldNamespace, "crate::builtin_script_rules()")),
  );
  for (const source of family) {
    check(source.after.archive, source.after);
    check(source.path, source.path === testPath ? current.test.after : source.after);
  }
  const helper = check(current.originalHelper.archive, current.originalHelper);
  regular(current.originalHelper.path);
  const composed = composeNoMutatingTestCorrectionHelper(helper);
  assert.deepEqual(read(current.originalHelper.path), composed);
  assert.deepEqual(fs.readFileSync(path.join(root, current.originalHelper.path)), composed);
  const oldLaws = check(current.originalFourLaws.archive, current.originalFourLaws);
  assert.deepEqual(check(current.originalFourLaws.path, current.originalFourLaws), oldLaws);
  return {
    manifest: current,
    paths: family.map(({ path: name }) => name),
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
  };
}

/** Only the corrected test body is projected after complete actual family proof. */
export function reviewedNoMutatingTestCorrectionReadView(root, read = reader(root)) {
  // Literal predecessor reads retain only their original historical authority.
  if (digest(read(testPath)) === originalTestSHA) return read;
  const { manifest } = authenticateNoMutatingTestCorrection(root, read);
  return (name, ...args) =>
    read(name === manifest.test.path ? manifest.test.before.archive : name, ...args);
}

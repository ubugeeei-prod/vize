import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../lint-default-correctness-reference.mjs";

const directory = "tests/_fixtures/differential/lint/props-deep-destructure-8090/source-custody";
const runtimeDirectory =
  "tests/_fixtures/differential/lint/runtime-prop-defaults-8564/source-custody";
const presetDirectory =
  "tests/_fixtures/differential/lint/setup-export-default-8090/source-custody";
export const deepPropsFamilyPaths = [
  "crates/vize_patina/src/rules/script/props_emits.rs",
  "crates/vize_patina/src/rules/script.rs",
  "crates/vize_patina/src/linter/script_rules.rs",
];
const reader = (root) => (name) => fs.readFileSync(path.join(root, name));
const matches = (bytes, identity) =>
  bytes.length === identity.bytes && sha256(bytes) === identity.sha256;
const blob = (bytes) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

/** A finite whole family; never substitute its source for an old performance cohort. */
export function authenticateDeepPropsSourceFamily(root, read = reader(root)) {
  const bytes = read(`${directory}/successor.json`);
  assert.equal(sha256(bytes), "de01ccf20b54abc69935b2ffa844705b1704bac8f83c07ee832c339ff9aaa161");
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, "vize.lint.props-deep-destructure-api-source-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.base, "edc003416dafd322aa6cd100dde488fb112a2abb");
  assert.equal(manifest.sourceCandidate, "33a66544ab5543b4eecbfb5f5dfd8bf1bce6297c");
  assert.deepEqual(manifest.changedPaths, deepPropsFamilyPaths);
  const runtimeBytes = read(`${runtimeDirectory}/successor.json`);
  assert.equal(
    sha256(runtimeBytes),
    "ea908f5de1c09c19ad59bf9f64bbbabccab676940f584fe754bb8e10f79828cb",
  );
  const runtime = JSON.parse(runtimeBytes);
  for (const source of runtime.sources) {
    for (const arm of [source.before, source.after]) {
      if (arm.absent) continue;
      const body = read(`${runtimeDirectory}/${arm.archive}`);
      assert(matches(body, arm), `original runtime archive changed: ${source.path}`);
      assert.equal(blob(body), arm.gitBlob);
    }
  }
  assert(matches(read(runtime.originalCustody.path), runtime.originalCustody));
  assert(matches(read(`${runtimeDirectory}/${runtime.originalLaw.path}`), runtime.originalLaw));
  const presetBytes = read(manifest.incomingFamily.path);
  assert(matches(presetBytes, manifest.incomingFamily));
  assert.equal(
    sha256(presetBytes),
    "7a612fc13dfe59fd50cfa31300980eacfa645e84a410cf6e04ea5777bc8d2e4b",
  );
  assert.equal(manifest.incomingFamily.path, `${presetDirectory}/successor.json`);
  const preset = JSON.parse(presetBytes);
  assert.deepEqual(JSON.parse(read(preset.incomingFamily.path)), runtime);
  assert.equal(preset.sources.length, 12);
  assert.equal(manifest.sources.length, 12);
  assert.deepEqual(
    manifest.sources.map(({ path }) => path),
    runtime.sources.map(({ path }) => path),
  );
  assert.equal(manifest.sources.filter(({ changed }) => changed).length, 3);
  let needsImportCorrection = false;
  for (const [index, source] of manifest.sources.entries()) {
    const previous = preset.sources[index];
    const original = runtime.sources[index].after;
    assert.equal(previous.path, source.path);
    assert.deepEqual(previous.before, {
      ...original,
      archive: path.posix.join(runtimeDirectory, original.archive),
    });
    const historical = read(previous.before.archive);
    const incoming = read(previous.after.archive);
    assert(matches(historical, previous.before));
    assert(matches(incoming, previous.after));
    if (previous.changed) {
      assert.equal(previous.path, "crates/vize_patina/src/linter/script_rules/registry/rules.rs");
      assert.equal(historical.toString().split(preset.change.before).length, 2);
      assert.equal(
        preset.change.after,
        preset.change.before.replace("OPT_IN_SCRIPT_PRESETS", "ESSENTIAL_SCRIPT_PRESETS"),
      );
      assert.deepEqual(
        incoming,
        Buffer.from(historical.toString().replace(preset.change.before, preset.change.after)),
      );
    } else assert.deepEqual(previous.after, previous.before);
    assert.deepEqual(source.before, previous.after);
    const after = read(source.after.archive);
    assert(matches(after, source.after), `new whole archive changed: ${source.path}`);
    assert.equal(blob(after), source.after.gitBlob);
    if (!matches(read(source.path), source.after)) {
      assert.equal(source.path, deepPropsFamilyPaths[2], `unknown current family: ${source.path}`);
      needsImportCorrection = true;
    }
    assert.equal(source.after.path, source.path);
    if (deepPropsFamilyPaths.includes(source.path)) {
      assert.equal(source.changed, true);
      let expected = incoming.toString();
      for (const transform of source.transforms) {
        assert.equal(expected.split(transform.before).length, 2);
        expected = expected.replace(transform.before, transform.after);
      }
      assert.deepEqual(after, Buffer.from(expected));
    } else {
      assert.equal(source.changed, false);
      assert.deepEqual(source.after, source.before);
      assert.deepEqual(source.transforms, []);
    }
  }
  const script = manifest.sources.find(({ path }) => path === deepPropsFamilyPaths[2]);
  const iterator = (body) => {
    const text = body.toString();
    const start = text.indexOf("fn active_builtin_script_rule_entries(");
    const end = text.indexOf("fn builtin_script_rule_entry(");
    assert(start >= 0 && end > start);
    return Buffer.from(text.slice(start, end).trimEnd() + "\n");
  };
  assert.deepEqual(iterator(read(script.before.archive)), iterator(read(script.after.archive)));
  assert(matches(iterator(read(script.after.archive)), runtime.retainedIterator));
  for (const arm of [script.before, script.after]) {
    assert.equal(
      read(arm.archive).toString().split(runtime.retainedDefaultDispatch.text).length,
      2,
    );
  }
  for (const arm of [manifest.producer.before, manifest.producer.after]) {
    const source = read(arm.archive);
    assert(matches(source, arm));
    assert.equal(blob(source), arm.gitBlob);
  }
  assert(matches(read(manifest.producer.after.path), manifest.producer.after));
  return needsImportCorrection
    ? { ...manifest, currentSuccessor: authenticateDeepPropsImportCorrection(root, manifest, read) }
    : manifest;
}

/** A whole twelve-body correction; the rejected import family stays immutable. */
export function authenticateDeepPropsImportCorrection(root, incoming, read = reader(root)) {
  const correctionDirectory = `${directory}/import-correction`;
  const bytes = read(`${correctionDirectory}/successor.json`);
  assert.equal(sha256(bytes), "39cf7ab7633378ab462c6af46443fa0e235b070d067db739eaa040760e3e996f");
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, "vize.lint.props-deep-destructure-import-source-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.candidate, "46c1f88e32a6c0f19c3de5ea6ee18aec91801223");
  assert.equal(manifest.originalFamily.path, `${directory}/successor.json`);
  assert(matches(read(manifest.originalFamily.path), manifest.originalFamily));
  assert.deepEqual(JSON.parse(read(manifest.originalFamily.path)), incoming);
  assert.deepEqual(manifest.changedPaths, [deepPropsFamilyPaths[2]]);
  assert.equal(manifest.sources.length, 12);
  assert.equal(manifest.sources.filter(({ changed }) => changed).length, 1);
  for (const [index, source] of manifest.sources.entries()) {
    assert.equal(source.path, incoming.sources[index].path);
    assert.deepEqual(source.before, incoming.sources[index].after);
    const before = read(source.before.archive);
    const after = read(source.after.archive);
    assert(matches(before, source.before));
    assert(matches(after, source.after));
    assert.equal(blob(before), source.before.gitBlob);
    assert.equal(blob(after), source.after.gitBlob);
    assert(matches(read(source.path), source.after), `unknown import successor: ${source.path}`);
    assert.equal(source.after.path, source.path);
    if (source.path === deepPropsFamilyPaths[2]) {
      assert.equal(source.changed, true);
      assert.equal(source.transforms.length, 2);
      let expected = before.toString();
      for (const transform of source.transforms) {
        assert.equal(expected.split(transform.before).length, 2);
        expected = expected.replace(transform.before, transform.after);
      }
      assert.deepEqual(after, Buffer.from(expected));
    } else {
      assert.equal(source.changed, false);
      assert.deepEqual(source.before, source.after);
      assert.deepEqual(source.transforms, []);
    }
  }
  return manifest;
}

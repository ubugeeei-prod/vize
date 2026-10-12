import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../lint-default-correctness-reference.mjs";
import {
  authenticateSetupExportPresetSourceFamily,
  setupExportRegistry,
} from "./setup-export-preset-source-custody.mjs";

const directory = "tests/_fixtures/differential/lint/runtime-prop-defaults-8564/source-custody";
const manifestHash = "ea908f5de1c09c19ad59bf9f64bbbabccab676940f584fe754bb8e10f79828cb";
const carriers = [
  "crates/vize_patina/src/linter/script_rules.rs",
  "crates/vize_patina/src/linter/script_rules/sfc.rs",
];
const sources = [
  "crates/vize_patina/src/rules/script/props_emits.rs",
  "crates/vize_patina/src/rules/script/props_emits/compiler_macro_bindings.rs",
  "crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator/setup.rs",
  "crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs",
  "crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default/runtime.rs",
  "crates/vize_patina/src/rules/script.rs",
  ...carriers,
  "crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default/typed.rs",
  "crates/vize_patina/src/linter/engine/sfc.rs",
  "crates/vize_patina/src/linter/engine.rs",
  "crates/vize_patina/src/linter/script_rules/registry/rules.rs",
];
const reader = (root) => (name) => fs.readFileSync(path.join(root, name));
const matches = (bytes, identity) =>
  bytes.length === identity.bytes && sha256(bytes) === identity.sha256;

function iterator(bytes) {
  const text = bytes.toString();
  const start = text.indexOf("fn active_builtin_script_rule_entries(");
  const end = text.indexOf("fn builtin_script_rule_entry(");
  assert(start >= 0 && end > start);
  return Buffer.from(text.slice(start, end).trimEnd() + "\n");
}

/** Original runtime family or its complete registry-only preset successor; old pins stay literal. */
export function authenticateRuntimeApiSourceFamily(root, read = reader(root)) {
  const bytes = read(`${directory}/successor.json`);
  assert.equal(sha256(bytes), manifestHash, "finite runtime source manifest changed");
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, "vize.lint.runtime-props-api-source-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.base, "6d7aaacc45e9d9d9cb0ed65052c61fa93b5b04c5");
  assert.equal(manifest.candidate, "029b7facb57d56e0c94780a4bc1f0820ceef996b");
  assert.deepEqual(
    manifest.changedCarriers.map(({ path }) => path),
    carriers,
  );
  assert.deepEqual(
    manifest.sources.map(({ path }) => path),
    sources,
  );
  assert.equal(manifest.sources.filter(({ changed }) => changed).length, 8);
  const custody = read(manifest.originalCustody.path);
  assert(matches(custody, manifest.originalCustody), "original source authority changed");
  const original = JSON.parse(custody);
  assert.deepEqual(
    original.currentSources.filter(({ path }) => carriers.includes(path)),
    manifest.changedCarriers,
  );
  assert(
    matches(read(`${directory}/${manifest.originalLaw.path}`), manifest.originalLaw),
    "original whole source law changed",
  );
  for (const source of manifest.sources) {
    const after = read(`${directory}/${source.after.archive}`);
    assert(matches(after, source.after), `authored after body changed: ${source.path}`);
    if (!matches(read(source.path), source.after)) {
      assert.equal(source.path, setupExportRegistry, `unqualified current body: ${source.path}`);
      authenticateSetupExportPresetSourceFamily(root, manifest, read);
    }
    assert.equal(source.path, source.after.path);
    if (source.before.absent) {
      assert.equal(source.changed, true);
      assert.deepEqual(source.before, { path: source.path, absent: true });
    } else {
      assert.equal(source.path, source.before.path);
      assert(
        matches(read(`${directory}/${source.before.archive}`), source.before),
        `authored before body changed: ${source.path}`,
      );
      assert.equal(source.changed, source.before.sha256 !== source.after.sha256);
    }
  }
  const script = manifest.sources.find(({ path }) => path === carriers[0]);
  const before = read(`${directory}/${script.before.archive}`);
  const after = read(`${directory}/${script.after.archive}`);
  assert.deepEqual(iterator(before), iterator(after), "existing rule-major order changed");
  assert(matches(iterator(after), manifest.retainedIterator));
  assert.equal(manifest.retainedIterator.sha256, original.retainedRuleIterator.sha256);
  assert.equal(manifest.retainedIterator.bytes, original.retainedRuleIterator.bytes);
  const ordinary = Buffer.from(manifest.retainedDefaultDispatch.text);
  assert(matches(ordinary, manifest.retainedDefaultDispatch));
  assert.equal(before.toString().split(ordinary.toString()).length, 2);
  assert.equal(after.toString().split(ordinary.toString()).length, 2);
  return manifest;
}

/** Preserve every original pin; admit only the two runtime carriers or exact preset registry. */
export function assertReservedApiOrderSource(root, source, read = reader(root)) {
  if (matches(read(source.path), source)) return "original";
  if (source.path === setupExportRegistry) {
    const incoming = authenticateRuntimeApiSourceFamily(root, read);
    const successor = authenticateSetupExportPresetSourceFamily(root, incoming, read);
    assert.deepEqual(source, successor.originalCarrier);
    return "setup-export-preset-successor";
  }
  assert(carriers.includes(source.path), `unqualified legacy source: ${source.path}`);
  const manifest = authenticateRuntimeApiSourceFamily(root, read);
  assert.deepEqual(
    source,
    manifest.changedCarriers.find(({ path }) => path === source.path),
  );
  return "runtime-props-successor";
}

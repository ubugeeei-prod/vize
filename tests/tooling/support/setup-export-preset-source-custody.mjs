import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../lint-default-correctness-reference.mjs";
import {
  authenticateDeepPropsSourceFamily,
  deepPropsFamilyPaths,
} from "./props-deep-source-custody.mjs";

export const setupExportRegistry = "crates/vize_patina/src/linter/script_rules/registry/rules.rs";
export const setupExportMembership =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";
const directory = "tests/_fixtures/differential/lint/setup-export-default-8090/source-custody";
const manifestHash = "7a612fc13dfe59fd50cfa31300980eacfa645e84a410cf6e04ea5777bc8d2e4b";
const reader = (root) => (name) => fs.readFileSync(path.join(root, name));
const matches = (bytes, identity) =>
  bytes.length === identity.bytes && sha256(bytes) === identity.sha256;

/** The complete preset family, with explicit authentication of its later depth successor. */
export function authenticateSetupExportPresetSourceFamily(root, incoming, read = reader(root)) {
  const bytes = read(`${directory}/successor.json`);
  assert.equal(sha256(bytes), manifestHash, "finite preset family changed");
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, "vize.lint.setup-export-default-api-source-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.base, "47b204f26839afb27af4a2f789eafc86295f0f81");
  const original = read(manifest.incomingFamily.path);
  assert(matches(original, manifest.incomingFamily));
  assert.equal(
    manifest.incomingFamily.sha256,
    "ea908f5de1c09c19ad59bf9f64bbbabccab676940f584fe754bb8e10f79828cb",
  );
  assert.deepEqual(JSON.parse(original), incoming, "incoming runtime family identity changed");
  assert.equal(manifest.sources.length, 12);
  assert.deepEqual(
    manifest.sources.map(({ path }) => path),
    incoming.sources.map(({ path }) => path),
  );
  assert.equal(manifest.sources.filter(({ changed }) => changed).length, 1);
  assert.equal(manifest.change.path, setupExportRegistry);
  assert.deepEqual(manifest.change.presets, [
    "essential",
    "happy-path",
    "ecosystem",
    "nuxt",
    "opinionated",
  ]);
  for (const [index, source] of manifest.sources.entries()) {
    const previous = incoming.sources[index].after;
    assert.deepEqual(source.before, {
      ...previous,
      archive: path.posix.join(path.posix.dirname(manifest.incomingFamily.path), previous.archive),
    });
    const before = read(source.before.archive);
    const after = read(source.after.archive);
    assert(matches(before, source.before), `incoming archive changed: ${source.path}`);
    assert(matches(after, source.after), `successor archive changed: ${source.path}`);
    if (!matches(read(source.path), source.after)) {
      assert(
        deepPropsFamilyPaths.includes(source.path),
        `unqualified current family: ${source.path}`,
      );
      authenticateDeepPropsSourceFamily(root, read);
    }
    assert.equal(source.path, source.after.path);
    if (source.path === setupExportRegistry) {
      assert.equal(source.changed, true);
      assert.equal(manifest.change.occurrences, 1);
      assert.equal(before.toString().split(manifest.change.before).length, 2);
      assert.equal(
        manifest.change.after,
        manifest.change.before.replace("OPT_IN_SCRIPT_PRESETS", "ESSENTIAL_SCRIPT_PRESETS"),
      );
      assert.deepEqual(
        after,
        Buffer.from(before.toString().replace(manifest.change.before, manifest.change.after)),
      );
      assert.deepEqual(manifest.originalCarrier, {
        path: source.path,
        sha256: source.before.sha256,
        bytes: source.before.bytes,
      });
    } else {
      assert.equal(source.changed, false);
      assert.deepEqual(
        source.after,
        source.before,
        `other incoming family changed: ${source.path}`,
      );
    }
  }
  return manifest;
}

/** Authenticate the whole new table, without granting old performance authority. */
export function authenticateSetupExportMembership(root, incoming, read = reader(root)) {
  authenticateSetupExportPresetSourceFamily(root, incoming, read);
  const bytes = read(`${directory}/membership-law.json`);
  assert.equal(
    sha256(bytes),
    "07f1abf28be1e0583a25f9c822beaaf546cec8c863603162ad04688833237dea",
    "finite default membership law changed",
  );
  const law = JSON.parse(bytes);
  assert.equal(law.schema, "vize.lint.setup-export-default-membership-law");
  assert.equal(law.version, 1);
  assert.equal(law.path, setupExportMembership);
  const before = read(`${directory}/original-preset-membership.snap.fixture`);
  const after = read(law.path);
  assert.equal(sha256(before), law.beforeSha256);
  assert.equal(sha256(after), law.afterSha256);
  const decode = (body) => JSON.parse(body.toString().split("---\n").at(-1));
  const expected = decode(before);
  assert.equal(law.changes.length, 5);
  for (const change of law.changes) {
    const values = expected[change.preset];
    assert.equal(values.length, change.beforeCount);
    assert.equal(values[change.index], change.next);
    values.splice(change.index, 0, "script/no-export-in-script-setup");
    assert.equal(values.length, change.afterCount);
  }
  assert.deepEqual(decode(after), expected);
  assert.deepEqual(expected.incremental, []);
  return law;
}

/** Authenticate the static marker interpretation before reading frozen gate history. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
export const staticListenerSource =
  "crates/vize_canon/src/virtual_ts/scope/component_events/generic_inference.rs";
export const staticListenerRecord =
  "tests/_fixtures/tooling/static-listener-prop-update-11815.json";
const recordSha256 = "9944123afc7253e4e154c209ea6f420d87d67db0326623546da355254c6781c6";
const emitGateSuccessor = "tests/_fixtures/tooling/emit-props-gate-current-source-20261105.json";
const emitGateSuccessorSha256 = "0de58b21496792302fc4139a2acea9fa24d66f6972e7361a9e5e31ffc30fb824";
type Identity = { bytes: number; sha256: string };
interface Transition {
  version: number;
  kind: string;
  nativeAcceptanceCredit: boolean;
  file: {
    path: string;
    before: Identity;
    after: Identity;
    edits: { before: string; after: string }[];
  };
}
interface EmitGateSuccessor {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  sources: Array<{
    path: string;
    hunks: Array<{ before: string; after: string }>;
  }>;
  dependencies: Array<Identity & { path: string }>;
}
const hash = (value: string) => createHash("sha256").update(value).digest("hex");
const identity = (value: string): Identity => ({
  bytes: Buffer.byteLength(value),
  sha256: hash(value),
});
export function beforeStaticListenerSource(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  if (path !== staticListenerSource) return current;
  const raw = read(staticListenerRecord);
  assert.equal(
    hash(raw),
    recordSha256,
    "selected emit gate producer: immutable static listener transition",
  );
  const record = JSON.parse(raw) as Transition;
  assert.equal(record.version, 1);
  assert.equal(record.kind, "exact-static-listener-prop-source-inverse");
  assert.equal(record.nativeAcceptanceCredit, false);
  assert.equal(record.file.path, path);
  if (identity(current).sha256 !== record.file.after.sha256) {
    assert.deepEqual(
      identity(current),
      {
        bytes: 18530,
        sha256: "33f33f57c1d8f4bcd5d61eea09cd8b5262e62bc02b3af2f52371407d58f10d1f",
      },
      "selected emit gate producer: integrated static listener source",
    );
    const successorRaw = read(emitGateSuccessor);
    assert.equal(
      hash(successorRaw),
      emitGateSuccessorSha256,
      "selected emit gate producer: immutable resolver-share transition",
    );
    const successor = JSON.parse(successorRaw) as EmitGateSuccessor;
    assert.equal(successor.schema, "vize.emitPropsGateCurrentSource.v1");
    assert.deepEqual(successor.credit, {
      compiled: false,
      native: false,
      performance: false,
    });
    const source = successor.sources.find((entry) => entry.path === path);
    assert.ok(source);
    assert.equal(source.hunks.length, 1);
    for (const dependency of successor.dependencies) {
      assert.deepEqual({ path: dependency.path, ...identity(read(dependency.path)) }, dependency);
    }
    const hunk = source.hunks[0]!;
    assert.equal(current.split(hunk.after).length, 2);
    assert.equal(current.includes(hunk.before), false);
    current = current.replace(hunk.after, hunk.before);
  }
  assert.deepEqual(
    identity(current),
    record.file.after,
    "selected emit gate producer: current static listener source",
  );
  for (const edit of record.file.edits) {
    assert.ok(edit.after);
    assert.equal(current.split(edit.after).length, 2);
    current = current.replace(edit.after, edit.before);
  }
  assert.deepEqual(
    identity(current),
    record.file.before,
    "selected emit gate producer: historical static listener source",
  );
  return current;
}

/** Recover the exact PR-base producer that still selected the loose marker per target. */
export function beforeIntegratedStaticListenerSource(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  if (path !== staticListenerSource) return current;
  assert.deepEqual(
    identity(current),
    {
      bytes: 18530,
      sha256: "33f33f57c1d8f4bcd5d61eea09cd8b5262e62bc02b3af2f52371407d58f10d1f",
    },
    "selected integrated static listener source",
  );
  const raw = read(staticListenerRecord);
  assert.equal(hash(raw), recordSha256, "immutable static listener transition");
  const record = JSON.parse(raw) as Transition;
  assert.equal(record.file.edits.length, 1);
  const edit = record.file.edits[0]!;
  assert.equal(current.split(edit.after).length, 2);
  current = current.replace(edit.after, edit.before);
  assert.deepEqual(
    identity(current),
    {
      bytes: 18240,
      sha256: "f231db13ede1284c8aa291ccd67bec0c50f7e68a510a1969b83e8e6c1fa70186",
    },
    "selected pre-static-fallback producer",
  );
  const previous =
    '    append!(\n        *ts,\n        "{indent}type {inferred_args} = {library} extends true ? (typeof {emit_props} extends {{ __ctx?: infer __X }} ? NonNullable<__X> extends {{ props: infer __P }} ? __P : {{}} : {{}}) extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : typeof {emit_props} extends {{ readonly __vizeLooseListener: infer __L extends unknown[] }} ? __L : typeof {emit_props} extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[];\\n",\n    );\n';
  const base =
    '    // A static component\'s loose sentinel means undeclared runtime arity,\n    // not a declared zero-argument event. Dynamic targets retain the empty\n    // contextual tuple; their contextless marker widens runtime args later.\n    let (loose_marker, loose_args) = if dynamic_component {\n        ("infer __L extends unknown[]", "__L")\n    } else {\n        ("unknown[]", "unknown[]")\n    };\n    append!(\n        *ts,\n        "{indent}type {inferred_args} = {library} extends true ? (typeof {emit_props} extends {{ __ctx?: infer __X }} ? NonNullable<__X> extends {{ props: infer __P }} ? __P : {{}} : {{}}) extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : typeof {emit_props} extends {{ readonly __vizeLooseListener: {loose_marker} }} ? {loose_args} : typeof {emit_props} extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[];\\n",\n    );\n';
  assert.equal(current.split(previous).length, 2, "selected PR-base listener transition");
  current = current.replace(previous, base);
  assert.deepEqual(
    identity(current),
    {
      bytes: 18626,
      sha256: "17fec691f4f92415b730dd4ce4d684056719c3045f31dc4b749e5c480aa2b771",
    },
    "selected PR-base static listener source",
  );
  return current;
}

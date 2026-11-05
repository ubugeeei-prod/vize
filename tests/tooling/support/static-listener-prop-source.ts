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
